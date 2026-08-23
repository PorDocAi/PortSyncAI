use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::models::attendances::{self, ApprovalStatus, GateStatus, WorkStatus};
use crate::models::employees::{self, Entity as Employees};
use crate::models::gate_terminals::{self, Entity as GateTerminals};
use crate::models::gate_verify_logs;
use crate::routes::attendances::{find_today_attendance, today};
use crate::utils::{
    AppState,
    auth::{AdminUser, GateTerminal, hash_token},
};

#[derive(Deserialize, vespera::Schema)]
pub struct GateVerifyRequest {
    /// 사원증 NFC UID (FR-D5)
    pub nfc_card_uid: String,
}

#[derive(Serialize, vespera::Schema)]
pub struct GateVerifyResponse {
    pub allowed: bool,
    pub employee_name: String,
    pub reason: String,
}

#[derive(Deserialize, vespera::Schema)]
pub struct IssueTerminalRequest {
    /// 게이트 식별자 (예: 북문, 남문)
    pub gate_id: String,
}

#[derive(Serialize, vespera::Schema)]
pub struct IssueTerminalResponse {
    /// 단말 고유번호
    pub terminal_id: i64,
    pub gate_id: String,
    /// 단말 토큰 평문 — 발급 시 단 한 번만 반환되며 다시 조회할 수 없다
    pub token: String,
}

/// 게이트 단말 자격증명 발급 (관리 권한)
/// 무작위 토큰을 생성하고 SHA-256 해시만 저장한다. 평문은 이 응답으로만 전달된다.
#[vespera::route(post, path = "/terminals", tags = ["gate"])]
pub async fn issue_gate_terminal(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<IssueTerminalRequest>,
) -> Result<(StatusCode, Json<IssueTerminalResponse>), StatusCode> {
    if req.gate_id.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let token = uuid::Uuid::new_v4().to_string();
    let terminal = gate_terminals::ActiveModel {
        gate_id: Set(req.gate_id.trim().to_string()),
        token_hash: Set(hash_token(&token)),
        ..Default::default()
    };
    let saved = terminal
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(IssueTerminalResponse {
            terminal_id: saved.terminal_id,
            gate_id: saved.gate_id,
            token,
        }),
    ))
}

/// 게이트 단말 폐기 (관리 권한) — 행을 지우지 않고 is_active만 false로 바꾼다 (소프트 삭제)
/// 이미 폐기된 단말도 멱등하게 200을 반환한다.
#[vespera::route(delete, path = "/terminals/{terminalid}", tags = ["gate"])]
pub async fn revoke_gate_terminal(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(terminal_id): Path<i64>,
) -> Result<Json<RevokeTerminalResponse>, StatusCode> {
    let terminal = GateTerminals::find_by_id(terminal_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if !terminal.is_active {
        let already = RevokeTerminalResponse {
            terminal_id: terminal.terminal_id,
            gate_id: terminal.gate_id,
            is_active: terminal.is_active,
        };
        return Ok(Json(already));
    }

    let mut active: gate_terminals::ActiveModel = terminal.clone().into();
    active.is_active = Set(false);
    let saved = active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RevokeTerminalResponse {
        terminal_id: saved.terminal_id,
        gate_id: saved.gate_id,
        is_active: saved.is_active,
    }))
}

#[derive(Serialize, vespera::Schema)]
pub struct RevokeTerminalResponse {
    pub terminal_id: i64,
    pub gate_id: String,
    pub is_active: bool,
}

/// 게이트 사원증 태깅 검증 (FR-D5, 데모 시나리오 1)
/// 게이트 단말 자격증명(Bearer 토큰)으로만 호출 가능하다.
/// (지침 확인 + 장비 완료 + 승인) 충족 시 통과 처리, 아니면 사유와 함께 차단.
/// 통과 판정은 (출근 건, 작업일) 유니크 제약으로 멱등 기록된다(AC-5).
#[vespera::route(post, path = "/verify", tags = ["gate"])]
pub async fn verify_gate(
    GateTerminal(terminal): GateTerminal,
    State(state): State<AppState>,
    Json(req): Json<GateVerifyRequest>,
) -> Result<Json<GateVerifyResponse>, StatusCode> {
    let employee = Employees::find()
        .filter(employees::Column::NfcCardUid.eq(&req.nfc_card_uid))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let Some(attendance) = find_today_attendance(&state.db, employee.employee_id).await? else {
        record_verify_log(
            &state,
            terminal.terminal_id,
            None,
            employee.employee_id,
            false,
            "오늘 출근 절차가 시작되지 않았습니다.",
            false,
        )
        .await?;
        return Ok(Json(GateVerifyResponse {
            allowed: false,
            employee_name: employee.name,
            reason: "오늘 출근 절차가 시작되지 않았습니다.".to_string(),
        }));
    };

    if attendance.gate_status == GateStatus::Passed {
        // 이미 PASSED인 재태깅: 상태 변경 없이 응답만 반환 — 통과 이벤트는 유니크 제약으로 1건 유지
        return Ok(Json(GateVerifyResponse {
            allowed: true,
            employee_name: employee.name,
            reason: "이미 통과 처리된 출근입니다.".to_string(),
        }));
    }

    // 차단 사유 수집 (FR-C4: 우회 경로 없음)
    // 단 APPROVED는 관리자가 사유와 함께 예외를 승인한 것이므로 미비 항목을 면제 (FR-E2)
    let mut reasons: Vec<&str> = Vec::new();
    match attendance.approval_status {
        ApprovalStatus::Approved => {}
        ApprovalStatus::Pending => reasons.push("관리자 승인 대기 중"),
        ApprovalStatus::Rejected => reasons.push("관리자 반려됨"),
        ApprovalStatus::NotRequired => {
            if !attendance.instruction_ack_completed {
                reasons.push("안전지침 미확인");
            }
            if !attendance.equipment_check_completed {
                reasons.push("필수 장비 확인 미완료");
            }
        }
    }

    // 작업중지는 승인 예외(APPROVED)로도 면제되지 않는 안전 정지 조치다 (시나리오 5)
    if attendance.work_status == WorkStatus::Stopped {
        reasons.push("작업중지");
    }

    if !reasons.is_empty() {
        let reason = reasons.join(", ");
        record_verify_log(
            &state,
            terminal.terminal_id,
            Some(attendance.attendance_id),
            employee.employee_id,
            false,
            &reason,
            false,
        )
        .await?;
        return Ok(Json(GateVerifyResponse {
            allowed: false,
            employee_name: employee.name,
            reason,
        }));
    }

    let attendance_id = attendance.attendance_id;
    let mut active: attendances::ActiveModel = attendance.into();
    active.gate_status = Set(GateStatus::Passed);
    active.gate_passed_at = Set(Some(chrono::Utc::now().into()));
    active.updated_at = Set(Some(chrono::Utc::now().into()));
    active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 통과 이벤트 기록 (is_pass_event = true) — (출근 건, 작업일) 유니크 제약이
    // 동시·반복 재태깅에서 중복 통과 이벤트를 차단한다 (AC-5)
    record_verify_log(
        &state,
        terminal.terminal_id,
        Some(attendance_id),
        employee.employee_id,
        true,
        "통과",
        true,
    )
    .await?;

    Ok(Json(GateVerifyResponse {
        allowed: true,
        employee_name: employee.name,
        reason: "통과".to_string(),
    }))
}

/// 검증 결과를 gate_verify_logs에 기록한다.
/// 허용 판정은 is_pass_event = true로 남겨 유니크 제약(uq_attendance_workdate_passed)이
/// 재태깅 시 중복 통과 이벤트를 차단한다 (AC-5).
#[allow(clippy::too_many_arguments)]
async fn record_verify_log(
    state: &AppState,
    terminal_id: i64,
    attendance_id: Option<i64>,
    employee_id: i64,
    allowed: bool,
    reason: &str,
    is_pass_event: bool,
) -> Result<(), StatusCode> {
    let log = gate_verify_logs::ActiveModel {
        attendance_id: Set(attendance_id),
        employee_id: Set(employee_id),
        terminal_id: Set(Some(terminal_id)),
        allowed: Set(allowed),
        reason: Set(reason.to_string()),
        work_date: Set(today()),
        is_pass_event: Set(is_pass_event),
        ..Default::default()
    };
    // 로그 기록 실패는 검증 결과에 영향을 주지 않는다.
    log.insert(&state.db).await.ok();
    Ok(())
}
