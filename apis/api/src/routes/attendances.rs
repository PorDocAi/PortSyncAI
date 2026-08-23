use std::collections::{HashMap, HashSet};

use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use vespera::axum::{Json, extract::State, http::StatusCode};

use crate::models::attendances::{self, ApprovalStatus, Entity as Attendances, GateStatus};
use crate::models::cargo_items::{self, Entity as CargoItems};
use crate::models::class_equipment_mappings::{
    self, Entity as ClassEquipmentMappings, RequirementLevel,
};
use crate::models::class_instruction_mappings::{self, Entity as ClassInstructionMappings};
use crate::models::employees::Entity as Employees;
use crate::models::equipment::{self, Entity as Equipment};
use crate::models::equipment_check_logs::{self, Entity as EquipmentCheckLogs};
use crate::models::equipment_types::{self, Entity as EquipmentTypes};
use crate::models::instruction_acknowledgements::{self, Entity as InstructionAcknowledgements};
use crate::models::safety_instruction_translations::{
    self, Entity as SafetyInstructionTranslations,
};
use crate::models::safety_instructions::{self, Entity as SafetyInstructions};
use crate::models::work_assignments::{self, EligibilityStatus, Entity as WorkAssignments};
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

pub async fn find_today_assignments(
    db: &sea_orm::DatabaseConnection,
    employee_id: i64,
) -> Result<Vec<work_assignments::Model>, StatusCode> {
    WorkAssignments::find()
        .filter(work_assignments::Column::EmployeeId.eq(employee_id))
        .filter(work_assignments::Column::WorkDate.eq(today()))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn assigned_cargo_class_ids(
    db: &sea_orm::DatabaseConnection,
    employee_id: i64,
) -> Result<Vec<i64>, StatusCode> {
    let cargo_item_ids: Vec<i64> = find_today_assignments(db, employee_id)
        .await?
        .into_iter()
        .filter_map(|assignment| assignment.cargo_item_id)
        .collect();
    if cargo_item_ids.is_empty() {
        return Ok(Vec::new());
    }

    let class_ids: HashSet<i64> = CargoItems::find()
        .filter(cargo_items::Column::CargoItemId.is_in(cargo_item_ids))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter_map(|cargo| cargo.dg_class_id)
        .collect();

    Ok(class_ids.into_iter().collect())
}

/// 작업자에게 오늘 배정된 화물의 위험물 Class에 연결된 활성 지침을 중복 없이 조회한다.
async fn required_instructions_for_employee_today(
    db: &sea_orm::DatabaseConnection,
    employee_id: i64,
) -> Result<Vec<safety_instructions::Model>, StatusCode> {
    let class_ids = assigned_cargo_class_ids(db, employee_id).await?;
    if class_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mappings = ClassInstructionMappings::find()
        .filter(class_instruction_mappings::Column::DgClassId.is_in(class_ids))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let instruction_ids: HashSet<i64> = mappings.into_iter().map(|m| m.instruction_id).collect();
    if instruction_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut instructions = SafetyInstructions::find()
        .filter(
            safety_instructions::Column::InstructionId
                .is_in(instruction_ids.into_iter().collect::<Vec<_>>()),
        )
        .filter(safety_instructions::Column::IsActive.eq(true))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    instructions.sort_by_key(|i| i.instruction_id);
    Ok(instructions)
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

    let assignments = find_today_assignments(&state.db, claims.sub).await?;
    if assignments.is_empty() {
        return Err(StatusCode::PRECONDITION_FAILED);
    }
    if assignments
        .iter()
        .any(|assignment| assignment.eligibility_status == EligibilityStatus::Excluded)
    {
        return Err(StatusCode::FORBIDDEN);
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

    // 확인할 지침이 없는 날은 형식적인 확인 절차를 만들지 않고 자동 완료한다.
    if required_instructions_for_employee_today(&state.db, claims.sub)
        .await?
        .is_empty()
    {
        let mut active: attendances::ActiveModel = saved.into();
        active.instruction_ack_completed = Set(true);
        active.updated_at = Set(Some(chrono::Utc::now().into()));
        let updated = active
            .update(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(Json(AttendanceResponse::from(updated)));
    }

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
    let required = required_instructions_for_employee_today(&state.db, claims.sub).await?;
    let instruction = required
        .iter()
        .find(|i| i.instruction_id == req.instruction_id)
        .ok_or(StatusCode::BAD_REQUEST)?;

    let employee = Employees::find_by_id(claims.sub)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let has_preferred_translation = SafetyInstructionTranslations::find()
        .filter(
            safety_instruction_translations::Column::InstructionId.eq(instruction.instruction_id),
        )
        .filter(
            safety_instruction_translations::Column::LanguageCode
                .eq(employee.preferred_language.clone()),
        )
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    let displayed_language = if has_preferred_translation {
        employee.preferred_language
    } else {
        "ko".to_string()
    };

    // 같은 버전의 지침은 재요청해도 로그를 중복 삽입하지 않는다.
    let existing = InstructionAcknowledgements::find()
        .filter(instruction_acknowledgements::Column::AttendanceId.eq(attendance.attendance_id))
        .filter(instruction_acknowledgements::Column::InstructionId.eq(instruction.instruction_id))
        .filter(instruction_acknowledgements::Column::InstructionVersion.eq(instruction.version))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if existing.is_none() {
        let ack = instruction_acknowledgements::ActiveModel {
            attendance_id: Set(attendance.attendance_id),
            employee_id: Set(claims.sub),
            instruction_id: Set(instruction.instruction_id),
            instruction_version: Set(instruction.version),
            language_code: Set(displayed_language),
            scrolled_to_end: Set(true),
            ..Default::default()
        };
        ack.insert(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let acknowledgements = InstructionAcknowledgements::find()
        .filter(instruction_acknowledgements::Column::AttendanceId.eq(attendance.attendance_id))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let acknowledged_versions: HashSet<(i64, i32)> = acknowledgements
        .into_iter()
        .map(|a| (a.instruction_id, a.instruction_version))
        .collect();
    let all_required_acknowledged = required
        .iter()
        .all(|i| acknowledged_versions.contains(&(i.instruction_id, i.version)));

    let mut active: attendances::ActiveModel = attendance.clone().into();
    active.instruction_ack_completed = Set(all_required_acknowledged);
    let mut updated_model = attendance;
    updated_model.instruction_ack_completed = all_required_acknowledged;
    promote_if_ready(&mut active, &updated_model);
    active.updated_at = Set(Some(chrono::Utc::now().into()));

    let saved = active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AttendanceResponse::from(saved)))
}

#[derive(Serialize, vespera::Schema)]
pub struct TodayInstructionItem {
    pub instruction_id: i64,
    pub version: i32,
    pub title: String,
    pub body: String,
    pub language_code: String,
    pub acknowledged: bool,
}

/// 오늘 입고 화물 Class에 연결된 지침을 작업자 모국어로 조회한다 (FR-B4/C1).
/// 번역본이 없으면 법적 원문의 기준 언어인 한국어로 대체한다.
#[vespera::route(get, path = "/today-instructions", tags = ["attendances"])]
pub async fn get_today_instructions(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<TodayInstructionItem>>, StatusCode> {
    let attendance = find_today_attendance(&state.db, claims.sub)
        .await?
        .ok_or(StatusCode::NOT_FOUND)?;
    let employee = Employees::find_by_id(claims.sub)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let instructions = required_instructions_for_employee_today(&state.db, claims.sub).await?;
    let ids: Vec<i64> = instructions.iter().map(|i| i.instruction_id).collect();

    let translations = if ids.is_empty() {
        Vec::new()
    } else {
        SafetyInstructionTranslations::find()
            .filter(safety_instruction_translations::Column::InstructionId.is_in(ids.clone()))
            .filter(
                safety_instruction_translations::Column::LanguageCode
                    .eq(employee.preferred_language.clone()),
            )
            .all(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    let translation_by_instruction: HashMap<i64, _> = translations
        .into_iter()
        .map(|t| (t.instruction_id, t))
        .collect();

    let acknowledgements = InstructionAcknowledgements::find()
        .filter(instruction_acknowledgements::Column::AttendanceId.eq(attendance.attendance_id))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let acknowledged_versions: HashSet<(i64, i32)> = acknowledgements
        .into_iter()
        .map(|a| (a.instruction_id, a.instruction_version))
        .collect();

    let items = instructions
        .into_iter()
        .map(|instruction| {
            let translation = translation_by_instruction.get(&instruction.instruction_id);
            TodayInstructionItem {
                instruction_id: instruction.instruction_id,
                version: instruction.version,
                title: translation
                    .map(|t| t.translated_title.clone())
                    .unwrap_or_else(|| instruction.title.clone()),
                body: translation
                    .map(|t| t.translated_body.clone())
                    .unwrap_or_else(|| instruction.summary.clone()),
                language_code: if translation.is_some() {
                    employee.preferred_language.clone()
                } else {
                    "ko".to_string()
                },
                acknowledged: acknowledged_versions
                    .contains(&(instruction.instruction_id, instruction.version)),
            }
        })
        .collect();

    Ok(Json(items))
}

/// 예외 승인 요청 (FR-E1): 미비 항목이 있을 때 관리자 승인을 요청
/// 미비 항목이 없으면 400 (승인이 필요 없는 상태)
#[vespera::route(post, path = "/request-approval", tags = ["attendances"])]
pub async fn request_approval(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<AttendanceResponse>, StatusCode> {
    let attendance = find_today_attendance(&state.db, claims.sub)
        .await?
        .ok_or(StatusCode::NOT_FOUND)?;

    if attendance.gate_status == GateStatus::Passed {
        return Err(StatusCode::CONFLICT);
    }
    if attendance.instruction_ack_completed && attendance.equipment_check_completed {
        return Err(StatusCode::BAD_REQUEST);
    }
    // 이미 요청된 상태면 그대로 반환 (멱등)
    if attendance.approval_status == ApprovalStatus::Pending {
        return Ok(Json(AttendanceResponse::from(attendance)));
    }

    let mut active: attendances::ActiveModel = attendance.into();
    active.approval_status = Set(ApprovalStatus::Pending);
    active.updated_at = Set(Some(chrono::Utc::now().into()));
    let saved = active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AttendanceResponse::from(saved)))
}

#[derive(Serialize, vespera::Schema)]
pub struct RequiredEquipmentItem {
    pub equipment_type_id: i64,
    pub name: String,
    /// REQUIRED | RECOMMENDED
    pub requirement_level: String,
    /// 이 출근 건에서 해당 종류 장비를 태깅 완료했는지
    pub satisfied: bool,
}

/// 작업자에게 당일 배정된 위험물 화물의 Class 기반 필수/권장 장비 목록과 충족 여부 산출 (FR-D1)
/// REQUIRED가 우선 — 같은 장비가 여러 Class에서 매핑되면 REQUIRED로 승격
async fn required_equipment_for_today(
    db: &sea_orm::DatabaseConnection,
    attendance_id: i64,
    employee_id: i64,
) -> Result<Vec<RequiredEquipmentItem>, StatusCode> {
    // 1) 이 작업자에게 오늘 배정된 화물의 Class 수집
    let class_ids = assigned_cargo_class_ids(db, employee_id).await?;
    if class_ids.is_empty() {
        return Ok(Vec::new());
    }

    // 2) Class → 장비 매핑 (장비종류별 요구 수준, REQUIRED 우선)
    let mappings = ClassEquipmentMappings::find()
        .filter(class_equipment_mappings::Column::DgClassId.is_in(class_ids))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut level_by_type: HashMap<i64, RequirementLevel> = HashMap::new();
    for m in &mappings {
        level_by_type
            .entry(m.equipment_type_id)
            .and_modify(|lv| {
                if m.requirement_level == RequirementLevel::Required {
                    *lv = RequirementLevel::Required;
                }
            })
            .or_insert(m.requirement_level.clone());
    }
    if level_by_type.is_empty() {
        return Ok(Vec::new());
    }

    // 3) 이 출근 건에서 태깅한 장비의 종류 집합
    let logs = EquipmentCheckLogs::find()
        .filter(equipment_check_logs::Column::AttendanceId.eq(attendance_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tagged_equipment_ids: Vec<i64> = logs.iter().map(|l| l.equipment_id).collect();
    let tagged_types: HashSet<i64> = if tagged_equipment_ids.is_empty() {
        HashSet::new()
    } else {
        Equipment::find()
            .filter(equipment::Column::EquipmentId.is_in(tagged_equipment_ids))
            .all(db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .iter()
            .map(|e| e.equipment_type_id)
            .collect()
    };

    // 4) 장비 종류명 조회 후 응답 구성
    let type_ids: Vec<i64> = level_by_type.keys().copied().collect();
    let names: HashMap<i64, String> = EquipmentTypes::find()
        .filter(equipment_types::Column::EquipmentTypeId.is_in(type_ids))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|t| (t.equipment_type_id, t.name))
        .collect();

    let mut items: Vec<RequiredEquipmentItem> = level_by_type
        .into_iter()
        .map(|(type_id, level)| RequiredEquipmentItem {
            equipment_type_id: type_id,
            name: names.get(&type_id).cloned().unwrap_or_default(),
            requirement_level: match level {
                RequirementLevel::Required => "REQUIRED".to_string(),
                RequirementLevel::Recommended => "RECOMMENDED".to_string(),
            },
            satisfied: tagged_types.contains(&type_id),
        })
        .collect();
    // REQUIRED 먼저, 그 안에서 종류 id 순으로 안정 정렬
    items.sort_by(|a, b| {
        b.requirement_level
            .cmp(&a.requirement_level)
            .then(a.equipment_type_id.cmp(&b.equipment_type_id))
    });
    Ok(items)
}

/// 당일 화물 기반 필수/권장 장비 목록 조회 (FR-D1)
#[vespera::route(get, path = "/required-equipment", tags = ["attendances"])]
pub async fn get_required_equipment(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<RequiredEquipmentItem>>, StatusCode> {
    let attendance = find_today_attendance(&state.db, claims.sub)
        .await?
        .ok_or(StatusCode::NOT_FOUND)?;
    let items =
        required_equipment_for_today(&state.db, attendance.attendance_id, claims.sub).await?;
    Ok(Json(items))
}

/// 장비 착용 완료 선언 (FR-D1): 당일 화물 Class 기반 필수(REQUIRED) 장비를
/// 전부 태깅해야 완료. 미충족 항목이 있으면 400.
#[vespera::route(post, path = "/equipment-complete", tags = ["attendances"])]
pub async fn complete_equipment_check(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<AttendanceResponse>, StatusCode> {
    let attendance = find_today_attendance(&state.db, claims.sub)
        .await?
        .ok_or(StatusCode::NOT_FOUND)?;

    let required =
        required_equipment_for_today(&state.db, attendance.attendance_id, claims.sub).await?;
    let all_required_satisfied = required
        .iter()
        .filter(|i| i.requirement_level == "REQUIRED")
        .all(|i| i.satisfied);
    if !all_required_satisfied {
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
