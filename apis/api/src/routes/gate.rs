use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use vespera::axum::{Json, extract::State, http::StatusCode};

use crate::models::attendances::{self, ApprovalStatus, GateStatus};
use crate::models::employees::{self, Entity as Employees};
use crate::routes::attendances::find_today_attendance;
use crate::utils::{AppState, auth::AuthUser};

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

/// 게이트 사원증 태깅 검증 (FR-D5, 데모 시나리오 1)
/// (지침 확인 + 장비 완료 + 승인) 충족 시 통과 처리, 아니면 사유와 함께 차단
/// 현재 게이트 단말 전용 자격증명은 분리하지 않았으며 로그인 JWT로 접근을 보호한다.
#[vespera::route(post, path = "/verify", tags = ["gate"])]
pub async fn verify_gate(
    _auth: AuthUser,
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
        return Ok(Json(GateVerifyResponse {
            allowed: false,
            employee_name: employee.name,
            reason: "오늘 출근 절차가 시작되지 않았습니다.".to_string(),
        }));
    };

    if attendance.gate_status == GateStatus::Passed {
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

    if !reasons.is_empty() {
        return Ok(Json(GateVerifyResponse {
            allowed: false,
            employee_name: employee.name,
            reason: reasons.join(", "),
        }));
    }

    let mut active: attendances::ActiveModel = attendance.into();
    active.gate_status = Set(GateStatus::Passed);
    active.gate_passed_at = Set(Some(chrono::Utc::now().into()));
    active.updated_at = Set(Some(chrono::Utc::now().into()));
    active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(GateVerifyResponse {
        allowed: true,
        employee_name: employee.name,
        reason: "통과".to_string(),
    }))
}
