use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use vespera::axum::{Json, extract::State, http::StatusCode};

use crate::models::attendances::{self, ApprovalStatus, Entity as Attendances, GateStatus};
use crate::models::instruction_acknowledgements;
use crate::models::safety_instructions::Entity as SafetyInstructions;
use crate::utils::{AppState, auth::AuthUser};

pub fn approval_as_str(s: &ApprovalStatus) -> &'static str {
    match s {
        ApprovalStatus::NotRequired => "NOT_REQUIRED",
        ApprovalStatus::Pending => "PENDING",
        ApprovalStatus::Approved => "APPROVED",
        ApprovalStatus::Rejected => "REJECTED",
    }
}

pub fn gate_as_str(s: &GateStatus) -> &'static str {
    match s {
        GateStatus::Blocked => "BLOCKED",
        GateStatus::Ready => "READY",
        GateStatus::Passed => "PASSED",
    }
}

/// 오늘 날짜 (서버 로컬 기준)
pub fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

/// 지침 확인 + 장비 완료 + 승인 상태가 모두 충족되면 READY로 승격
pub fn promote_if_ready(active: &mut attendances::ActiveModel, model: &attendances::Model) {
    let approval_ok = matches!(
        model.approval_status,
        ApprovalStatus::NotRequired | ApprovalStatus::Approved
    );
    if model.gate_status == GateStatus::Blocked
        && model.instruction_ack_completed
        && model.equipment_check_completed
        && approval_ok
    {
        active.gate_status = Set(GateStatus::Ready);
    }
}

#[derive(Serialize, vespera::Schema)]
pub struct AttendanceResponse {
    pub attendance_id: i64,
    pub employee_id: i64,
    pub work_date: String,
    pub instruction_ack_completed: bool,
    pub equipment_check_completed: bool,
    pub approval_status: String,
    pub gate_status: String,
    pub gate_passed_at: Option<String>,
}

impl From<attendances::Model> for AttendanceResponse {
    fn from(m: attendances::Model) -> Self {
        Self {
            attendance_id: m.attendance_id,
            employee_id: m.employee_id,
            work_date: m.work_date.to_string(),
            instruction_ack_completed: m.instruction_ack_completed,
            equipment_check_completed: m.equipment_check_completed,
            approval_status: approval_as_str(&m.approval_status).to_string(),
            gate_status: gate_as_str(&m.gate_status).to_string(),
            gate_passed_at: m.gate_passed_at.map(|t| t.to_rfc3339()),
        }
    }
}

pub async fn find_today_attendance(
    db: &sea_orm::DatabaseConnection,
    employee_id: i64,
) -> Result<Option<attendances::Model>, StatusCode> {
    Attendances::find()
        .filter(attendances::Column::EmployeeId.eq(employee_id))
        .filter(attendances::Column::WorkDate.eq(today()))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// 오늘 출근 절차 시작 (이미 시작했으면 기존 레코드 반환 — 멱등)
#[vespera::route(post, tags = ["attendances"])]
pub async fn start_attendance(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<AttendanceResponse>, StatusCode> {
    if let Some(existing) = find_today_attendance(&state.db, claims.sub).await? {
        return Ok(Json(AttendanceResponse::from(existing)));
    }
    let new_attendance = attendances::ActiveModel {
        employee_id: Set(claims.sub),
        work_date: Set(today()),
        ..Default::default()
    };
    let saved = new_attendance
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AttendanceResponse::from(saved)))
}

/// 오늘 내 출근 상태 조회
#[vespera::route(get, path = "/today", tags = ["attendances"])]
pub async fn get_today_attendance(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<AttendanceResponse>, StatusCode> {
    let attendance = find_today_attendance(&state.db, claims.sub)
        .await?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(AttendanceResponse::from(attendance)))
}

#[derive(Deserialize, vespera::Schema)]
pub struct AckRequest {
    pub instruction_id: i64,
    pub language_code: String,
    /// 팝업 최하단 스크롤 완료 여부 (FR-C2 — false면 확인 불가)
    pub scrolled_to_end: bool,
}

/// 안전지침 확인 (FR-C2/C3): 스크롤 완료 시에만 인지 로그 기록
#[vespera::route(post, path = "/ack", tags = ["attendances"])]
pub async fn acknowledge_instruction(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<AckRequest>,
) -> Result<Json<AttendanceResponse>, StatusCode> {
    if !req.scrolled_to_end {
        return Err(StatusCode::BAD_REQUEST);
    }
    let attendance = find_today_attendance(&state.db, claims.sub)
        .await?
        .ok_or(StatusCode::NOT_FOUND)?;
    let instruction = SafetyInstructions::find_by_id(req.instruction_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // 인지 로그 (append-only — 면책 증빙, FR-G1)
    let ack = instruction_acknowledgements::ActiveModel {
        attendance_id: Set(attendance.attendance_id),
        employee_id: Set(claims.sub),
        instruction_id: Set(instruction.instruction_id),
        instruction_version: Set(instruction.version),
        language_code: Set(req.language_code),
        scrolled_to_end: Set(true),
        ..Default::default()
    };
    ack.insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut active: attendances::ActiveModel = attendance.clone().into();
    active.instruction_ack_completed = Set(true);
    let mut updated_model = attendance;
    updated_model.instruction_ack_completed = true;
    promote_if_ready(&mut active, &updated_model);
    active.updated_at = Set(Some(chrono::Utc::now().into()));

    let saved = active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AttendanceResponse::from(saved)))
}

/// 장비 착용 완료 선언 (오늘 태깅 기록이 1건 이상 있어야 함)
/// TODO: 화물 API 연동 후 당일 화물 Class 기반 필수 장비 전체 충족 검증으로 강화 (FR-D1)
#[vespera::route(post, path = "/equipment-complete", tags = ["attendances"])]
pub async fn complete_equipment_check(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<AttendanceResponse>, StatusCode> {
    use crate::models::equipment_check_logs::{self, Entity as EquipmentCheckLogs};

    let attendance = find_today_attendance(&state.db, claims.sub)
        .await?
        .ok_or(StatusCode::NOT_FOUND)?;

    let tagged = EquipmentCheckLogs::find()
        .filter(equipment_check_logs::Column::AttendanceId.eq(attendance.attendance_id))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if tagged.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut active: attendances::ActiveModel = attendance.clone().into();
    active.equipment_check_completed = Set(true);
    let mut updated_model = attendance;
    updated_model.equipment_check_completed = true;
    promote_if_ready(&mut active, &updated_model);
    active.updated_at = Set(Some(chrono::Utc::now().into()));

    let saved = active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AttendanceResponse::from(saved)))
}
