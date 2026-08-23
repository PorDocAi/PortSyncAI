use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, ExprTrait, QueryFilter, QueryOrder, QuerySelect, Statement, Value,
};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::models::attendances::{self, ApprovalStatus, GateStatus, WorkStatus};
use crate::models::education_completions::{self, Entity as EducationCompletions};
use crate::models::education_target_rules::{self, Entity as EducationTargetRules};
use crate::models::employees::{self, Entity as Employees};
use crate::models::gate_events::{self, GateDecision};
use crate::models::gate_terminals::{self, Entity as GateTerminals};
use crate::models::gate_verify_logs;
use crate::models::v2_work_assignments::{self, Entity as V2WorkAssignments};
use crate::models::work_assignment_equipment::{self, Entity as WorkAssignmentEquipment};

use crate::models::work_stops::{self, Entity as WorkStops, WorkStopStatus};
use crate::routes::attendances::{find_today_attendance, today};
use crate::utils::{
    AppState,
    auth::{AdminUser, GateTerminal, hash_token},
};

#[derive(Deserialize, vespera::Schema)]
pub struct GateVerifyRequest {
    /// 사원증 NFC UID (FR-D5)
    pub nfc_card_uid: String,
    /// v2 작업 배정 준비도를 함께 검증할 때 사용하는 선택 식별자
    pub v2_work_assignment_id: Option<i64>,
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

    let v2_readiness = if req.v2_work_assignment_id.is_some() {
        Some(evaluate_v2_readiness(&state.db, &employee, req.v2_work_assignment_id).await?)
    } else {
        None
    };

    let (allowed, reason, v2_reason_codes) = match v2_readiness {
        Some(readiness) if !readiness.is_empty() => (false, readiness.join(", "), readiness),
        _ => {
            if attendance.gate_status == GateStatus::Passed {
                // 이미 PASSED인 재태깅: 상태 변경 없이 응답만 반환 — 통과 이벤트는 유니크 제약으로 1건 유지
                record_gate_event(
                    &state,
                    terminal.terminal_id,
                    terminal.gate_id.clone(),
                    req.v2_work_assignment_id,
                    employee.employee_id,
                    true,
                    Vec::new(),
                    "이미 통과 처리된 출근입니다.".to_string(),
                )
                .await?;
                return Ok(Json(GateVerifyResponse {
                    allowed: true,
                    employee_name: employee.name,
                    reason: "이미 통과 처리된 출근입니다.".to_string(),
                }));
            }
            (true, "통과".to_string(), Vec::new())
        }
    };

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

    if !reasons.is_empty() || !allowed {
        let reason = if !v2_reason_codes.is_empty() {
            v2_reason_codes.join(", ")
        } else {
            reasons.join(", ")
        };
        let mut codes = v2_reason_codes.clone();
        if attendance.work_status == WorkStatus::Stopped {
            codes.push("WORK_STOPPED".to_string());
        }
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
        record_gate_event(
            &state,
            terminal.terminal_id,
            terminal.gate_id.clone(),
            req.v2_work_assignment_id,
            employee.employee_id,
            false,
            codes,
            reason.clone(),
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
        &reason,
        true,
    )
    .await?;

    record_gate_event(
        &state,
        terminal.terminal_id,
        terminal.gate_id.clone(),
        req.v2_work_assignment_id,
        employee.employee_id,
        allowed,
        v2_reason_codes,
        reason.clone(),
    )
    .await?;

    Ok(Json(GateVerifyResponse {
        allowed: true,
        employee_name: employee.name,
        reason,
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

/// v2 작업 배정 준비도를 검사한다. 레거시 출근 차단과 독립적인 안정 코드만 반환한다.
async fn evaluate_v2_readiness(
    db: &DatabaseConnection,
    employee: &employees::Model,
    work_assignment_id: Option<i64>,
) -> Result<Vec<String>, StatusCode> {
    let Some(assignment) =
        V2WorkAssignments::find_by_id(work_assignment_id.expect("caller checks ID"))
            .one(db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(vec!["ASSIGNMENT_NOT_FOUND".to_string()]);
    };

    let mut reason_codes = Vec::new();
    let open_stop = WorkStops::find()
        .filter(
            work_stops::Column::WorkAssignmentId
                .eq(assignment.work_assignment_id)
                .and(work_stops::Column::Status.eq(WorkStopStatus::Open)),
        )
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if open_stop.is_some() {
        reason_codes.push("WORK_STOPPED".to_string());
    }

    let snapshot_ids: Vec<i64> = db
        .query_all_raw(Statement::from_sql_and_values(
            db.get_database_backend(),
            "SELECT work_ppe_requirement_snapshot_id FROM work_ppe_requirement_snapshots WHERE work_id = ?",
            vec![Value::BigInt(Some(assignment.work_id))],
        ))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|row| row.try_get_by_index(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR))
        .collect::<Result<_, _>>()?;
    if snapshot_ids.is_empty() {
        reason_codes.push("PPE_INCOMPLETE".to_string());
    } else {
        for snapshot_id in snapshot_ids {
            let allocation = WorkAssignmentEquipment::find()
                .filter(
                    work_assignment_equipment::Column::WorkAssignmentId
                        .eq(assignment.work_assignment_id)
                        .and(
                            work_assignment_equipment::Column::RequirementSnapshotId
                                .eq(snapshot_id),
                        ),
                )
                .one(db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if allocation.is_none() {
                reason_codes.push("PPE_INCOMPLETE".to_string());
            }
        }
    }

    let required_course_ids: Vec<i64> = EducationTargetRules::find()
        .columns([
            education_target_rules::Column::EducationCourseId,
            education_target_rules::Column::EducationTargetRuleId,
        ])
        .filter(
            education_target_rules::Column::WorkTypeId
                .eq(Some(assignment_work_type_id(db, &assignment).await?))
                .and(education_target_rules::Column::IsRequired.eq(true))
                .and(education_target_rules::Column::IsActive.eq(true)),
        )
        .order_by_asc(education_target_rules::Column::EducationCourseId)
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|rule| rule.education_course_id)
        .collect();

    for course_id in required_course_ids {
        let completion = EducationCompletions::find()
            .filter(
                education_completions::Column::EmployeeId
                    .eq(employee.employee_id)
                    .and(education_completions::Column::EducationCourseId.eq(course_id))
                    .and(
                        education_completions::Column::ExpiresAt
                            .is_null()
                            .or(education_completions::Column::ExpiresAt.gt(now())),
                    ),
            )
            .order_by_desc(education_completions::Column::CompletedAt)
            .one(db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if completion.is_none() {
            reason_codes.push("EDUCATION_MISSING".to_string());
        }
    }

    Ok(reason_codes)
}

async fn assignment_work_type_id(
    db: &DatabaseConnection,
    assignment: &v2_work_assignments::Model,
) -> Result<i64, StatusCode> {
    crate::models::works::Entity::find_by_id(assignment.work_id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|work| work.work_type_id)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

fn now() -> sea_orm::entity::prelude::DateTimeWithTimeZone {
    chrono::Utc::now().fixed_offset()
}

#[allow(clippy::too_many_arguments)]
async fn record_gate_event(
    state: &AppState,
    terminal_id: i64,
    gate_id: String,
    work_assignment_id: Option<i64>,
    employee_id: i64,
    allowed: bool,
    reason_codes: Vec<String>,
    response_reason: String,
) -> Result<(), StatusCode> {
    let (primary_code, codes_json): (String, serde_json::Value) = if reason_codes.is_empty() {
        ("PASS".to_string(), serde_json::json!(["PASS"]))
    } else {
        let json =
            serde_json::to_value(&reason_codes).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        (reason_codes[0].clone(), json)
    };
    let response_json = serde_json::json!({
        "allowed": allowed,
        "reason": response_reason,
    });
    let event = gate_events::ActiveModel {
        terminal_id: Set(Some(terminal_id)),
        gate_id: Set(gate_id),
        work_assignment_id: Set(work_assignment_id),
        employee_id: Set(employee_id),
        decision: Set(if allowed {
            GateDecision::Pass
        } else {
            GateDecision::Block
        }),
        reason_code: Set(primary_code),
        reason_codes: Set(codes_json),
        decision_input_versions: Set(serde_json::Value::Null),
        http_status: Set(200),
        response_json: Set(response_json),
        occurred_at: Set(now()),
        ..Default::default()
    };
    event
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}
