use std::{collections::HashSet, sync::OnceLock};

use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseTransaction,
    DbBackend, EntityTrait, QueryFilter, QueryOrder, QueryResult, Statement, TransactionTrait,
    TryInsertResult, Value,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use vespera::axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::models::{
    employees::{EmployeeStatus, Entity as Employees},
    equipment_check_events::{self, Entity as EquipmentCheckEvents},
    equipment_profiles::{
        Entity as EquipmentProfiles, EquipmentLifecycleStatus, EquipmentOwnershipType,
    },
    equipment_tag_tokens::{self, Entity as EquipmentTagTokens},
    equipment_types::Entity as EquipmentTypes,
    shared_equipment_claims::{self, Entity as SharedEquipmentClaims},
    v2_work_assignments::{self, Entity as V2WorkAssignments, V2WorkAssignmentStatus},
    work_assignment_equipment::{self, Entity as WorkAssignmentEquipment},
    works::{Entity as Works, WorkLifecycleStatus},
};
use crate::utils::{
    AppState,
    auth::{AdminUser, AuthUser, hash_token},
};

const TAG_TOKEN_PREFIX: &str = "eqt_";
static TOKEN_ISSUE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

/// v2 장비 태깅 요청. 작업자·장비·종류는 요청에서 받지 않고 JWT, 배정, 토큰에서
/// 각각 확인한다.
#[derive(Clone, Debug, Deserialize, Serialize, vespera::Schema)]
pub struct TagRequest {
    pub work_assignment_id: i64,
    /// 서버가 발급한 대소문자 구분 불투명 토큰. 공백 제거·정규화를 하지 않는다.
    pub tag_token: String,
    /// RFC3339 클라이언트 감사 시각. 작업 선택이나 서버 시각 대체에 사용하지 않는다.
    pub client_scanned_at: String,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, vespera::Schema)]
pub struct EquipmentTypeSummary {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, vespera::Schema)]
pub struct EquipmentSummary {
    pub id: i64,
    pub asset_number: Option<String>,
    #[serde(rename = "type")]
    pub equipment_type: EquipmentTypeSummary,
    pub ownership: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, vespera::Schema)]
pub struct RequirementSummary {
    pub id: i64,
    pub category: String,
    pub satisfied: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, vespera::Schema)]
pub struct ChecklistItem {
    pub id: i64,
    pub category: String,
    pub satisfied: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, vespera::Schema)]
pub struct RequirementProgress {
    pub required: u64,
    pub satisfied: u64,
    pub complete: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, vespera::Schema)]
pub struct TagResponse {
    pub accepted: bool,
    pub equipment: Option<EquipmentSummary>,
    pub requirement: Option<RequirementSummary>,
    pub progress: RequirementProgress,
    pub checklist: Vec<ChecklistItem>,
    pub reason_code: String,
    pub message: Option<String>,
}

#[derive(Clone, Debug)]
struct ScanResult {
    status: StatusCode,
    response: TagResponse,
}

#[derive(Clone, Debug)]
struct ScanDecision {
    status: StatusCode,
    response: TagResponse,
    equipment_profile_id: Option<i64>,
}

#[derive(Debug, Deserialize, vespera::Schema)]
pub struct IssueEquipmentTagTokenRequest {
    pub equipment_profile_id: i64,
}

#[derive(Debug, Serialize, vespera::Schema)]
pub struct IssueEquipmentTagTokenResponse {
    pub equipment_tag_token_id: i64,
    pub equipment_profile_id: i64,
    /// 평문은 발급/회전 응답으로만 한 번 반환한다.
    pub token: String,
    pub token_uri: String,
    pub ndef_mime_type: String,
}

#[derive(Debug, Serialize, vespera::Schema)]
pub struct RevokeEquipmentTagTokenResponse {
    pub equipment_tag_token_id: i64,
    pub equipment_profile_id: i64,
    pub is_active: bool,
}

#[derive(Debug, Serialize, vespera::Schema)]
pub struct TokenAdminError {
    pub reason_code: String,
}

fn token_admin_error(status: StatusCode, reason_code: &str) -> (StatusCode, Json<TokenAdminError>) {
    (
        status,
        Json(TokenAdminError {
            reason_code: reason_code.to_string(),
        }),
    )
}

/// 최소 128비트 이상의 서버 난수. 두 UUID를 이어 붙여 UUID v4의 고정 비트까지
/// 고려해도 240비트 이상이 되도록 한다.
fn new_tag_token() -> String {
    format!(
        "{TAG_TOKEN_PREFIX}{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    )
}

fn status_code_for_lifecycle(status: &EquipmentLifecycleStatus) -> Option<&'static str> {
    match status {
        EquipmentLifecycleStatus::Available => None,
        EquipmentLifecycleStatus::Blocked => Some("EQUIPMENT_BLOCKED"),
        EquipmentLifecycleStatus::Damaged => Some("EQUIPMENT_DAMAGED"),
        EquipmentLifecycleStatus::Lost => Some("EQUIPMENT_LOST"),
        EquipmentLifecycleStatus::Replaced => Some("EQUIPMENT_REPLACED"),
    }
}

fn lifecycle_message(reason_code: &str) -> &'static str {
    match reason_code {
        "EQUIPMENT_BLOCKED" => "장비가 관리자에 의해 차단되었습니다.",
        "EQUIPMENT_DAMAGED" => "손상된 장비는 사용할 수 없습니다.",
        "EQUIPMENT_LOST" => "분실된 장비는 사용할 수 없습니다.",
        "EQUIPMENT_REPLACED" => "교체된 장비는 사용할 수 없습니다.",
        _ => "장비를 사용할 수 없습니다.",
    }
}

fn ownership_as_str(ownership: &EquipmentOwnershipType) -> &'static str {
    match ownership {
        EquipmentOwnershipType::Personal => "PERSONAL",
        EquipmentOwnershipType::Shared => "SHARED",
    }
}

fn lifecycle_as_str(status: &EquipmentLifecycleStatus) -> &'static str {
    match status {
        EquipmentLifecycleStatus::Available => "AVAILABLE",
        EquipmentLifecycleStatus::Blocked => "BLOCKED",
        EquipmentLifecycleStatus::Damaged => "DAMAGED",
        EquipmentLifecycleStatus::Lost => "LOST",
        EquipmentLifecycleStatus::Replaced => "REPLACED",
    }
}

fn progress_zero() -> RequirementProgress {
    RequirementProgress {
        required: 0,
        satisfied: 0,
        complete: true,
    }
}

fn response_for(
    accepted: bool,
    reason_code: &str,
    message: Option<&str>,
    equipment: Option<EquipmentSummary>,
    requirement: Option<RequirementSummary>,
    progress: RequirementProgress,
    checklist: Vec<ChecklistItem>,
) -> TagResponse {
    TagResponse {
        accepted,
        equipment,
        requirement,
        progress,
        checklist,
        reason_code: reason_code.to_string(),
        message: message.map(str::to_string),
    }
}

fn unpersisted_error(status: StatusCode, reason_code: &str, message: &str) -> ScanResult {
    ScanResult {
        status,
        response: response_for(
            false,
            reason_code,
            Some(message),
            None,
            None,
            progress_zero(),
            Vec::new(),
        ),
    }
}

fn parse_request(
    req: &TagRequest,
) -> Result<(DateTime<FixedOffset>, Uuid, String), Box<ScanResult>> {
    let scanned_at = DateTime::parse_from_rfc3339(&req.client_scanned_at).map_err(|_| {
        Box::new(unpersisted_error(
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            "client_scanned_at은 RFC3339 형식이어야 합니다.",
        ))
    })?;
    let idempotency_key = Uuid::parse_str(&req.idempotency_key).map_err(|_| {
        Box::new(unpersisted_error(
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            "idempotency_key은 UUID 형식이어야 합니다.",
        ))
    })?;
    let fingerprint = {
        let token_hash = hash_token(&req.tag_token);
        let canonical = format!(
            "v2|{}|{}|{}|{}",
            req.work_assignment_id, token_hash, req.client_scanned_at, req.idempotency_key
        );
        format!("{:x}", Sha256::digest(canonical.as_bytes()))
    };
    Ok((scanned_at, idempotency_key, fingerprint))
}

fn replay_result(event: &equipment_check_events::Model) -> Result<ScanResult, sea_orm::DbErr> {
    let response: TagResponse = serde_json::from_value(event.response_json.clone())
        .map_err(|error| sea_orm::DbErr::Json(error.to_string()))?;
    let status =
        StatusCode::from_u16(event.http_status as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    Ok(ScanResult { status, response })
}

fn idempotency_reused() -> ScanResult {
    unpersisted_error(
        StatusCode::CONFLICT,
        "IDEMPOTENCY_KEY_REUSED",
        "같은 멱등키로 다른 요청을 재사용할 수 없습니다.",
    )
}

async fn finish_event(
    tx: &DatabaseTransaction,
    mut event: equipment_check_events::Model,
    decision: ScanDecision,
) -> Result<ScanResult, sea_orm::DbErr> {
    let response_json = serde_json::to_value(&decision.response)
        .map_err(|error| sea_orm::DbErr::Json(error.to_string()))?;
    let mut active: equipment_check_events::ActiveModel = event.clone().into();
    active.equipment_profile_id = Set(decision.equipment_profile_id);
    active.http_status = Set(decision.status.as_u16() as i16);
    active.accepted = Set(decision.response.accepted);
    active.reason_code = Set(decision.response.reason_code.clone());
    active.response_json = Set(response_json);
    event = active.update(tx).await?;

    // FR-D1 게이트 판정(attendances.required-equipment)과 통합:
    // 승인된 태깅은 당일 출근의 장비 확인 로그에도 적립한다.
    if decision.response.accepted {
        if let Some(profile_id) = decision.equipment_profile_id {
            use crate::models::attendances::{self, Entity as Attendances};
            let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
            let today_attendance = Attendances::find()
                .filter(attendances::Column::EmployeeId.eq(event.employee_id))
                .filter(attendances::Column::WorkDate.eq(&today))
                .one(tx)
                .await?
                .map(|a| a.attendance_id);
            if let Some(attendance_id) = today_attendance {
                use crate::models::equipment_check_logs::{self, Entity as EquipmentCheckLogs};
                use crate::models::equipment_profiles::Entity as EquipmentProfiles;
                // legacy_equipment가 연결된 경우 그 ID를, 아니면 profile ID를 사용한다.
                let legacy_equipment_id = EquipmentProfiles::find_by_id(profile_id)
                    .one(tx)
                    .await?
                    .and_then(|pr| pr.legacy_equipment_id)
                    .unwrap_or(profile_id);
                let already = EquipmentCheckLogs::find()
                    .filter(equipment_check_logs::Column::AttendanceId.eq(attendance_id))
                    .filter(equipment_check_logs::Column::EquipmentId.eq(legacy_equipment_id))
                    .one(tx)
                    .await?;
                if already.is_none() {
                    equipment_check_logs::ActiveModel {
                        attendance_id: Set(attendance_id),
                        employee_id: Set(event.employee_id),
                        equipment_id: Set(legacy_equipment_id),
                        work_date: Set(chrono::Utc::now().date_naive()),
                        ..Default::default()
                    }
                    .insert(tx)
                    .await?;
                }
            }
        }
    }

    Ok(ScanResult {
        status: StatusCode::from_u16(event.http_status as u16)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        response: decision.response,
    })
}

#[derive(Clone, Debug)]
struct Snapshot {
    work_ppe_requirement_snapshot_id: i64,
    equipment_type_id: i64,
    category: String,
}

fn snapshot_from_row(row: &QueryResult) -> Result<Snapshot, sea_orm::DbErr> {
    Ok(Snapshot {
        work_ppe_requirement_snapshot_id: row.try_get("", "work_ppe_requirement_snapshot_id")?,
        equipment_type_id: row.try_get("", "equipment_type_id")?,
        category: row.try_get("", "category")?,
    })
}

async fn load_snapshots(
    db: &DatabaseTransaction,
    work_id: i64,
) -> Result<Vec<Snapshot>, sea_orm::DbErr> {
    let backend = db.get_database_backend();
    let (placeholder, values) = if backend == DbBackend::Postgres {
        ("$1", vec![Value::BigInt(Some(work_id))])
    } else {
        ("?", vec![Value::BigInt(Some(work_id))])
    };
    let sql = format!(
        "SELECT work_ppe_requirement_snapshot_id, equipment_type_id, category FROM work_ppe_requirement_snapshots WHERE work_id = {placeholder} ORDER BY work_ppe_requirement_snapshot_id"
    );
    db.query_all_raw(Statement::from_sql_and_values(backend, sql, values))
        .await?
        .iter()
        .map(snapshot_from_row)
        .collect()
}

async fn progress_for(
    db: &DatabaseTransaction,
    assignment: &v2_work_assignments::Model,
    snapshots: &[Snapshot],
) -> Result<(RequirementProgress, Vec<ChecklistItem>), sea_orm::DbErr> {
    let allocations = WorkAssignmentEquipment::find()
        .filter(
            work_assignment_equipment::Column::WorkAssignmentId.eq(assignment.work_assignment_id),
        )
        .all(db)
        .await?;
    let mut satisfied_snapshot_ids = HashSet::new();
    for allocation in allocations {
        let Some(profile) = EquipmentProfiles::find_by_id(allocation.equipment_profile_id)
            .one(db)
            .await?
        else {
            continue;
        };
        if profile.status != EquipmentLifecycleStatus::Available {
            continue;
        }
        match profile.ownership_type {
            EquipmentOwnershipType::Personal
                if profile.owner_employee_id != Some(assignment.employee_id) =>
            {
                continue;
            }
            EquipmentOwnershipType::Shared => {
                let claim = SharedEquipmentClaims::find()
                    .filter(
                        shared_equipment_claims::Column::EquipmentProfileId
                            .eq(profile.equipment_profile_id),
                    )
                    .one(db)
                    .await?;
                if claim
                    .as_ref()
                    .is_none_or(|claim| claim.work_assignment_id != assignment.work_assignment_id)
                {
                    continue;
                }
            }
            EquipmentOwnershipType::Personal => {}
        }
        satisfied_snapshot_ids.insert(allocation.requirement_snapshot_id);
    }

    let checklist = snapshots
        .iter()
        .map(|snapshot| ChecklistItem {
            id: snapshot.work_ppe_requirement_snapshot_id,
            category: snapshot.category.clone(),
            satisfied: satisfied_snapshot_ids.contains(&snapshot.work_ppe_requirement_snapshot_id),
        })
        .collect::<Vec<_>>();
    let satisfied = checklist.iter().filter(|item| item.satisfied).count() as u64;
    let required = snapshots.len() as u64;
    Ok((
        RequirementProgress {
            required,
            satisfied,
            complete: satisfied == required,
        },
        checklist,
    ))
}

async fn release_terminal_claim_if_needed(
    tx: &DatabaseTransaction,
    claim: &shared_equipment_claims::Model,
) -> Result<bool, sea_orm::DbErr> {
    let assignment = V2WorkAssignments::find_by_id(claim.work_assignment_id)
        .one(tx)
        .await?;
    let Some(assignment) = assignment else {
        SharedEquipmentClaims::delete_by_id(claim.shared_equipment_claim_id)
            .exec(tx)
            .await?;
        return Ok(true);
    };
    let work = Works::find_by_id(assignment.work_id).one(tx).await?;
    let terminal = matches!(
        assignment.status,
        V2WorkAssignmentStatus::Completed | V2WorkAssignmentStatus::Cancelled
    ) || work.as_ref().is_some_and(|work| {
        matches!(
            work.status,
            WorkLifecycleStatus::Completed | WorkLifecycleStatus::Cancelled
        )
    });
    if terminal {
        SharedEquipmentClaims::delete_by_id(claim.shared_equipment_claim_id)
            .exec(tx)
            .await?;
    }
    Ok(terminal)
}

async fn process_scan(
    tx: &DatabaseTransaction,
    claims: &crate::utils::jwt::Claims,
    req: &TagRequest,
    scanned_at: DateTime<FixedOffset>,
    idempotency_key: Uuid,
    fingerprint: String,
) -> Result<ScanResult, sea_orm::DbErr> {
    if let Some(event) = EquipmentCheckEvents::find()
        .filter(equipment_check_events::Column::EmployeeId.eq(claims.sub))
        .filter(equipment_check_events::Column::IdempotencyKey.eq(idempotency_key))
        .one(tx)
        .await?
    {
        return if event.request_fingerprint == fingerprint {
            replay_result(&event)
        } else {
            Ok(idempotency_reused())
        };
    }

    let Some(employee) = Employees::find_by_id(claims.sub).one(tx).await? else {
        return Ok(unpersisted_error(
            StatusCode::FORBIDDEN,
            "EMPLOYEE_INACTIVE",
            "사용할 수 없는 작업자입니다.",
        ));
    };
    if employee.status != EmployeeStatus::Active {
        return Ok(unpersisted_error(
            StatusCode::FORBIDDEN,
            "EMPLOYEE_INACTIVE",
            "사용할 수 없는 작업자입니다.",
        ));
    }

    let Some(assignment) = V2WorkAssignments::find_by_id(req.work_assignment_id)
        .one(tx)
        .await?
    else {
        return Ok(unpersisted_error(
            StatusCode::NOT_FOUND,
            "WORK_ASSIGNMENT_NOT_FOUND",
            "작업 배정을 찾을 수 없습니다.",
        ));
    };
    if assignment.employee_id != claims.sub {
        return Ok(unpersisted_error(
            StatusCode::NOT_FOUND,
            "WORK_ASSIGNMENT_NOT_FOUND",
            "작업 배정을 찾을 수 없습니다.",
        ));
    }

    let Some(work) = Works::find_by_id(assignment.work_id).one(tx).await? else {
        return Ok(unpersisted_error(
            StatusCode::CONFLICT,
            "WORK_ASSIGNMENT_NOT_ACTIVE",
            "작업이 현재 태깅 가능한 상태가 아닙니다.",
        ));
    };
    let snapshots = load_snapshots(tx, work.work_id).await?;
    let (initial_progress, initial_checklist) = progress_for(tx, &assignment, &snapshots).await?;

    let terminal_assignment = matches!(
        assignment.status,
        V2WorkAssignmentStatus::Completed | V2WorkAssignmentStatus::Cancelled
    );
    let terminal_work = matches!(
        work.status,
        WorkLifecycleStatus::Completed | WorkLifecycleStatus::Cancelled
    );
    if terminal_assignment || terminal_work {
        let terminal_claims = SharedEquipmentClaims::find()
            .filter(
                shared_equipment_claims::Column::WorkAssignmentId.eq(assignment.work_assignment_id),
            )
            .all(tx)
            .await?;
        for claim in terminal_claims {
            SharedEquipmentClaims::delete_by_id(claim.shared_equipment_claim_id)
                .exec(tx)
                .await?;
        }
    }

    let assignment_active = matches!(
        assignment.status,
        V2WorkAssignmentStatus::Selected | V2WorkAssignmentStatus::Active
    ) && assignment.selected_at.is_some();
    let work_active = matches!(work.status, WorkLifecycleStatus::Active);
    let placeholder_response = response_for(
        false,
        "PROCESSING",
        None,
        None,
        None,
        initial_progress.clone(),
        initial_checklist.clone(),
    );
    let placeholder = equipment_check_events::ActiveModel {
        employee_id: Set(claims.sub),
        work_assignment_id: Set(assignment.work_assignment_id),
        equipment_profile_id: Set(None),
        idempotency_key: Set(idempotency_key),
        request_fingerprint: Set(fingerprint.clone()),
        http_status: Set(StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i16),
        accepted: Set(false),
        reason_code: Set("PROCESSING".to_string()),
        response_json: Set(serde_json::to_value(placeholder_response)
            .map_err(|error| sea_orm::DbErr::Json(error.to_string()))?),
        client_scanned_at: Set(scanned_at),
        ..Default::default()
    };
    let inserted = EquipmentCheckEvents::insert(placeholder)
        .on_conflict_do_nothing_on([
            equipment_check_events::Column::EmployeeId,
            equipment_check_events::Column::IdempotencyKey,
        ])
        .exec(tx)
        .await?;
    let event = match inserted {
        TryInsertResult::Inserted(result) => {
            EquipmentCheckEvents::find_by_id(result.last_insert_id)
                .one(tx)
                .await?
                .ok_or_else(|| {
                    sea_orm::DbErr::RecordNotFound("equipment check event".to_string())
                })?
        }
        TryInsertResult::Conflicted => {
            let Some(existing) = EquipmentCheckEvents::find()
                .filter(equipment_check_events::Column::EmployeeId.eq(claims.sub))
                .filter(equipment_check_events::Column::IdempotencyKey.eq(idempotency_key))
                .one(tx)
                .await?
            else {
                return Err(sea_orm::DbErr::RecordNotFound(
                    "conflicted equipment check event".to_string(),
                ));
            };
            return if existing.request_fingerprint == fingerprint {
                replay_result(&existing)
            } else {
                Ok(idempotency_reused())
            };
        }
        TryInsertResult::Empty => {
            return Err(sea_orm::DbErr::RecordNotInserted);
        }
    };

    let finish = |decision: ScanDecision| async move { finish_event(tx, event, decision).await };

    if !assignment_active || !work_active {
        return finish(ScanDecision {
            status: StatusCode::CONFLICT,
            response: response_for(
                false,
                "WORK_ASSIGNMENT_NOT_ACTIVE",
                Some("작업이 현재 태깅 가능한 상태가 아닙니다."),
                None,
                None,
                initial_progress,
                initial_checklist,
            ),
            equipment_profile_id: None,
        })
        .await;
    }

    let token_hash = hash_token(&req.tag_token);
    let Some(tag_token) = EquipmentTagTokens::find()
        .filter(equipment_tag_tokens::Column::TagTokenHash.eq(token_hash))
        .filter(equipment_tag_tokens::Column::IsActive.eq(true))
        .one(tx)
        .await?
    else {
        return finish(ScanDecision {
            status: StatusCode::NOT_FOUND,
            response: response_for(
                false,
                "TAG_NOT_REGISTERED",
                Some("등록되지 않았거나 비활성화된 태그입니다."),
                None,
                None,
                initial_progress,
                initial_checklist,
            ),
            equipment_profile_id: None,
        })
        .await;
    };

    let Some(profile) = EquipmentProfiles::find_by_id(tag_token.equipment_profile_id)
        .one(tx)
        .await?
    else {
        return finish(ScanDecision {
            status: StatusCode::NOT_FOUND,
            response: response_for(
                false,
                "TAG_NOT_REGISTERED",
                Some("등록되지 않았거나 비활성화된 태그입니다."),
                None,
                None,
                initial_progress,
                initial_checklist,
            ),
            equipment_profile_id: None,
        })
        .await;
    };
    let Some(equipment_type) = EquipmentTypes::find_by_id(profile.equipment_type_id)
        .one(tx)
        .await?
    else {
        return Err(sea_orm::DbErr::RecordNotFound("equipment type".to_string()));
    };
    let equipment = EquipmentSummary {
        id: profile.equipment_profile_id,
        asset_number: profile.asset_number.clone(),
        equipment_type: EquipmentTypeSummary {
            id: equipment_type.equipment_type_id,
            name: equipment_type.name,
        },
        ownership: ownership_as_str(&profile.ownership_type).to_string(),
        status: lifecycle_as_str(&profile.status).to_string(),
    };

    if let Some(reason_code) = status_code_for_lifecycle(&profile.status) {
        return finish(ScanDecision {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            response: response_for(
                false,
                reason_code,
                Some(lifecycle_message(reason_code)),
                Some(equipment),
                None,
                initial_progress,
                initial_checklist,
            ),
            equipment_profile_id: Some(profile.equipment_profile_id),
        })
        .await;
    }
    if profile.ownership_type == EquipmentOwnershipType::Personal
        && profile.owner_employee_id != Some(claims.sub)
    {
        return finish(ScanDecision {
            status: StatusCode::FORBIDDEN,
            response: response_for(
                false,
                "PERSONAL_EQUIPMENT_OWNER_MISMATCH",
                Some("개인 장비의 소유자만 태깅할 수 있습니다."),
                Some(equipment),
                None,
                initial_progress,
                initial_checklist,
            ),
            equipment_profile_id: Some(profile.equipment_profile_id),
        })
        .await;
    }

    if snapshots.is_empty() {
        return finish(ScanDecision {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            response: response_for(
                false,
                "PPE_REQUIREMENTS_UNAVAILABLE",
                Some("작업 보호구 요구조건을 확인할 수 없습니다."),
                Some(equipment),
                None,
                initial_progress,
                initial_checklist,
            ),
            equipment_profile_id: Some(profile.equipment_profile_id),
        })
        .await;
    }

    let existing_allocation = WorkAssignmentEquipment::find()
        .filter(
            work_assignment_equipment::Column::WorkAssignmentId.eq(assignment.work_assignment_id),
        )
        .filter(
            work_assignment_equipment::Column::EquipmentProfileId.eq(profile.equipment_profile_id),
        )
        .one(tx)
        .await?;
    if existing_allocation.is_some() {
        let (progress, checklist) = progress_for(tx, &assignment, &snapshots).await?;
        return finish(ScanDecision {
            status: StatusCode::OK,
            response: response_for(
                true,
                "TAG_ALREADY_ACCEPTED",
                None,
                Some(equipment),
                None,
                progress,
                checklist,
            ),
            equipment_profile_id: Some(profile.equipment_profile_id),
        })
        .await;
    }

    let Some(snapshot) = snapshots
        .iter()
        .find(|snapshot| snapshot.equipment_type_id == profile.equipment_type_id)
    else {
        return finish(ScanDecision {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            response: response_for(
                false,
                "PPE_REQUIREMENT_MISMATCH",
                Some("이 장비 종류는 현재 작업의 보호구 요구조건과 일치하지 않습니다."),
                Some(equipment),
                None,
                initial_progress,
                initial_checklist,
            ),
            equipment_profile_id: Some(profile.equipment_profile_id),
        })
        .await;
    };

    let already_satisfied = WorkAssignmentEquipment::find()
        .filter(
            work_assignment_equipment::Column::WorkAssignmentId.eq(assignment.work_assignment_id),
        )
        .filter(
            work_assignment_equipment::Column::RequirementSnapshotId
                .eq(snapshot.work_ppe_requirement_snapshot_id),
        )
        .one(tx)
        .await?;
    if already_satisfied.is_some() {
        return finish(ScanDecision {
            status: StatusCode::CONFLICT,
            response: response_for(
                false,
                "PPE_REQUIREMENT_ALREADY_SATISFIED",
                Some("해당 보호구 요구조건은 이미 충족되었습니다."),
                Some(equipment),
                Some(RequirementSummary {
                    id: snapshot.work_ppe_requirement_snapshot_id,
                    category: snapshot.category.clone(),
                    satisfied: true,
                }),
                initial_progress,
                initial_checklist,
            ),
            equipment_profile_id: Some(profile.equipment_profile_id),
        })
        .await;
    }

    if profile.ownership_type == EquipmentOwnershipType::Shared {
        let mut claim_owned_by_assignment = false;
        if let Some(claim) = SharedEquipmentClaims::find()
            .filter(
                shared_equipment_claims::Column::EquipmentProfileId
                    .eq(profile.equipment_profile_id),
            )
            .one(tx)
            .await?
        {
            if release_terminal_claim_if_needed(tx, &claim).await? {
                // The terminal claim was removed below; a new claim can be acquired.
            } else if claim.work_assignment_id == assignment.work_assignment_id {
                // A concurrent scan for this same assignment may have won the claim.
                // It is not a cross-assignment conflict; the allocation check below
                // will turn this into TAG_ALREADY_ACCEPTED when appropriate.
                claim_owned_by_assignment = true;
            } else {
                let (progress, checklist) = progress_for(tx, &assignment, &snapshots).await?;
                return finish(ScanDecision {
                    status: StatusCode::CONFLICT,
                    response: response_for(
                        false,
                        "SHARED_EQUIPMENT_IN_USE",
                        Some("공용 장비가 다른 활성 작업에서 사용 중입니다."),
                        Some(equipment),
                        Some(RequirementSummary {
                            id: snapshot.work_ppe_requirement_snapshot_id,
                            category: snapshot.category.clone(),
                            satisfied: false,
                        }),
                        progress,
                        checklist,
                    ),
                    equipment_profile_id: Some(profile.equipment_profile_id),
                })
                .await;
            }
        }

        if !claim_owned_by_assignment {
            let claim_inserted =
                SharedEquipmentClaims::insert(shared_equipment_claims::ActiveModel {
                    work_assignment_id: Set(assignment.work_assignment_id),
                    equipment_profile_id: Set(profile.equipment_profile_id),
                    ..Default::default()
                })
                .on_conflict_do_nothing_on([shared_equipment_claims::Column::EquipmentProfileId])
                .exec(tx)
                .await?;
            if matches!(claim_inserted, TryInsertResult::Conflicted) {
                let Some(claim) = SharedEquipmentClaims::find()
                    .filter(
                        shared_equipment_claims::Column::EquipmentProfileId
                            .eq(profile.equipment_profile_id),
                    )
                    .one(tx)
                    .await?
                else {
                    return Err(sea_orm::DbErr::RecordNotFound(
                        "shared equipment claim".to_string(),
                    ));
                };
                if claim.work_assignment_id != assignment.work_assignment_id {
                    let (progress, checklist) = progress_for(tx, &assignment, &snapshots).await?;
                    return finish(ScanDecision {
                        status: StatusCode::CONFLICT,
                        response: response_for(
                            false,
                            "SHARED_EQUIPMENT_IN_USE",
                            Some("공용 장비가 다른 활성 작업에서 사용 중입니다."),
                            Some(equipment),
                            Some(RequirementSummary {
                                id: snapshot.work_ppe_requirement_snapshot_id,
                                category: snapshot.category.clone(),
                                satisfied: false,
                            }),
                            progress,
                            checklist,
                        ),
                        equipment_profile_id: Some(profile.equipment_profile_id),
                    })
                    .await;
                }
            }
        }
    }

    let allocation_inserted =
        WorkAssignmentEquipment::insert(work_assignment_equipment::ActiveModel {
            work_assignment_id: Set(assignment.work_assignment_id),
            equipment_profile_id: Set(profile.equipment_profile_id),
            requirement_snapshot_id: Set(snapshot.work_ppe_requirement_snapshot_id),
            ..Default::default()
        })
        .on_conflict_do_nothing_on([
            work_assignment_equipment::Column::WorkAssignmentId,
            work_assignment_equipment::Column::EquipmentProfileId,
        ])
        .exec(tx)
        .await?;
    if matches!(allocation_inserted, TryInsertResult::Conflicted) {
        let (progress, checklist) = progress_for(tx, &assignment, &snapshots).await?;
        return finish(ScanDecision {
            status: StatusCode::OK,
            response: response_for(
                true,
                "TAG_ALREADY_ACCEPTED",
                None,
                Some(equipment),
                Some(RequirementSummary {
                    id: snapshot.work_ppe_requirement_snapshot_id,
                    category: snapshot.category.clone(),
                    satisfied: true,
                }),
                progress,
                checklist,
            ),
            equipment_profile_id: Some(profile.equipment_profile_id),
        })
        .await;
    }

    let (progress, checklist) = progress_for(tx, &assignment, &snapshots).await?;
    finish(ScanDecision {
        status: StatusCode::CREATED,
        response: response_for(
            true,
            "TAG_ACCEPTED",
            None,
            Some(equipment),
            Some(RequirementSummary {
                id: snapshot.work_ppe_requirement_snapshot_id,
                category: snapshot.category.clone(),
                satisfied: true,
            }),
            progress,
            checklist,
        ),
        equipment_profile_id: Some(profile.equipment_profile_id),
    })
    .await
}

/// 장비 NFC 태깅. v2 sidecar의 불투명 토큰만 장비 식별에 사용한다.
#[vespera::route(post, tags = ["equipment_checks"])]
pub async fn tag_equipment(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<TagRequest>,
) -> Result<(StatusCode, Json<TagResponse>), (StatusCode, Json<TagResponse>)> {
    let (scanned_at, idempotency_key, fingerprint) =
        parse_request(&req).map_err(|result| (result.status, Json(result.response)))?;
    let tx = state.db.begin().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                unpersisted_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "서버 오류가 발생했습니다.",
                )
                .response,
            ),
        )
    })?;
    let result = process_scan(&tx, &claims, &req, scanned_at, idempotency_key, fingerprint).await;
    match result {
        Ok(result) => {
            tx.commit().await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(
                        unpersisted_error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "INTERNAL_ERROR",
                            "서버 오류가 발생했습니다.",
                        )
                        .response,
                    ),
                )
            })?;
            Ok((result.status, Json(result.response)))
        }
        Err(_) => {
            let _ = tx.rollback().await;
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    unpersisted_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        "서버 오류가 발생했습니다.",
                    )
                    .response,
                ),
            ))
        }
    }
}

async fn issue_token_for_profile_locked(
    state: &AppState,
    admin_id: i64,
    equipment_profile_id: i64,
) -> Result<(StatusCode, Json<IssueEquipmentTagTokenResponse>), (StatusCode, Json<TokenAdminError>)>
{
    EquipmentProfiles::find_by_id(equipment_profile_id)
        .one(&state.db)
        .await
        .map_err(|_| token_admin_error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"))?
        .ok_or_else(|| token_admin_error(StatusCode::NOT_FOUND, "EQUIPMENT_PROFILE_NOT_FOUND"))?;
    let existing = EquipmentTagTokens::find()
        .filter(equipment_tag_tokens::Column::EquipmentProfileId.eq(equipment_profile_id))
        .filter(equipment_tag_tokens::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(|_| token_admin_error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"))?;
    if existing.is_some() {
        return Err(token_admin_error(
            StatusCode::CONFLICT,
            "TAG_TOKEN_ALREADY_ACTIVE",
        ));
    }

    let token = new_tag_token();
    let saved = equipment_tag_tokens::ActiveModel {
        equipment_profile_id: Set(equipment_profile_id),
        tag_token_hash: Set(hash_token(&token)),
        is_active: Set(true),
        issued_by_id: Set(admin_id),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(|_| token_admin_error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"))?;
    Ok((
        StatusCode::CREATED,
        Json(IssueEquipmentTagTokenResponse {
            equipment_tag_token_id: saved.equipment_tag_token_id,
            equipment_profile_id: saved.equipment_profile_id,
            token_uri: format!("portsync://equipment/{token}"),
            token,
            ndef_mime_type: "application/vnd.portsync.equipment+json".to_string(),
        }),
    ))
}

async fn issue_token_for_profile(
    state: &AppState,
    admin_id: i64,
    equipment_profile_id: i64,
) -> Result<(StatusCode, Json<IssueEquipmentTagTokenResponse>), (StatusCode, Json<TokenAdminError>)>
{
    let lock = TOKEN_ISSUE_LOCK.get_or_init(|| tokio::sync::Mutex::new(()));
    let _guard = lock.lock().await;
    issue_token_for_profile_locked(state, admin_id, equipment_profile_id).await
}

/// 관리자 장비 프로필 태그 토큰 발급. 평문은 이 응답에서만 반환하고 DB에는 SHA-256만 저장한다.
#[vespera::route(post, path = "/tag-tokens", tags = ["equipment_checks"])]
pub async fn issue_equipment_tag_token(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<IssueEquipmentTagTokenRequest>,
) -> Result<(StatusCode, Json<IssueEquipmentTagTokenResponse>), (StatusCode, Json<TokenAdminError>)>
{
    issue_token_for_profile(&state, claims.sub, req.equipment_profile_id).await
}

/// 프로필 경로를 사용하는 관리자 발급 별칭.
#[vespera::route(post, path = "/{equipment_profile_id}/tag-token", tags = ["equipment_checks"])]
pub async fn issue_equipment_tag_token_for_profile(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Path(equipment_profile_id): Path<i64>,
) -> Result<(StatusCode, Json<IssueEquipmentTagTokenResponse>), (StatusCode, Json<TokenAdminError>)>
{
    issue_token_for_profile(&state, claims.sub, equipment_profile_id).await
}

async fn revoke_token_by_id(
    state: &AppState,
    admin_id: i64,
    token_id: i64,
) -> Result<Json<RevokeEquipmentTagTokenResponse>, (StatusCode, Json<TokenAdminError>)> {
    let token = EquipmentTagTokens::find_by_id(token_id)
        .one(&state.db)
        .await
        .map_err(|_| token_admin_error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"))?
        .ok_or_else(|| token_admin_error(StatusCode::NOT_FOUND, "TAG_TOKEN_NOT_FOUND"))?;
    if token.is_active {
        let mut active: equipment_tag_tokens::ActiveModel = token.clone().into();
        active.is_active = Set(false);
        active.deactivated_by_id = Set(Some(admin_id));
        active.deactivated_at = Set(Some(Utc::now().fixed_offset()));
        active
            .update(&state.db)
            .await
            .map_err(|_| token_admin_error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"))?;
    }
    Ok(Json(RevokeEquipmentTagTokenResponse {
        equipment_tag_token_id: token.equipment_tag_token_id,
        equipment_profile_id: token.equipment_profile_id,
        is_active: false,
    }))
}

/// 관리자 태그 토큰 폐기. 행은 남기고 활성 플래그만 끈다.
#[vespera::route(delete, path = "/tag-tokens/{tokenid}", tags = ["equipment_checks"])]
pub async fn revoke_equipment_tag_token(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Path(token_id): Path<i64>,
) -> Result<Json<RevokeEquipmentTagTokenResponse>, (StatusCode, Json<TokenAdminError>)> {
    revoke_token_by_id(&state, claims.sub, token_id).await
}

/// 프로필 경로를 사용하는 관리자 폐기 별칭. 해당 프로필의 활성 토큰을 모두 폐기한다.
#[vespera::route(delete, path = "/{equipment_profile_id}/tag-token", tags = ["equipment_checks"])]
pub async fn revoke_equipment_tag_token_for_profile(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Path(equipment_profile_id): Path<i64>,
) -> Result<Json<RevokeEquipmentTagTokenResponse>, (StatusCode, Json<TokenAdminError>)> {
    let token = EquipmentTagTokens::find()
        .filter(equipment_tag_tokens::Column::EquipmentProfileId.eq(equipment_profile_id))
        .filter(equipment_tag_tokens::Column::IsActive.eq(true))
        .order_by_asc(equipment_tag_tokens::Column::EquipmentTagTokenId)
        .one(&state.db)
        .await
        .map_err(|_| token_admin_error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"))?
        .ok_or_else(|| token_admin_error(StatusCode::NOT_FOUND, "TAG_TOKEN_NOT_FOUND"))?;
    revoke_token_by_id(&state, claims.sub, token.equipment_tag_token_id).await
}

/// 기존 활성 토큰을 폐기하고 새 토큰을 발급한다.
#[vespera::route(post, path = "/tag-tokens/{equipment_profile_id}/rotate", tags = ["equipment_checks"])]
pub async fn rotate_equipment_tag_token(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Path(equipment_profile_id): Path<i64>,
) -> Result<(StatusCode, Json<IssueEquipmentTagTokenResponse>), (StatusCode, Json<TokenAdminError>)>
{
    let lock = TOKEN_ISSUE_LOCK.get_or_init(|| tokio::sync::Mutex::new(()));
    let _guard = lock.lock().await;
    let active_tokens = EquipmentTagTokens::find()
        .filter(equipment_tag_tokens::Column::EquipmentProfileId.eq(equipment_profile_id))
        .filter(equipment_tag_tokens::Column::IsActive.eq(true))
        .all(&state.db)
        .await
        .map_err(|_| token_admin_error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"))?;
    for token in active_tokens {
        let mut active: equipment_tag_tokens::ActiveModel = token.into();
        active.is_active = Set(false);
        active.deactivated_by_id = Set(Some(claims.sub));
        active.deactivated_at = Set(Some(Utc::now().fixed_offset()));
        active
            .update(&state.db)
            .await
            .map_err(|_| token_admin_error(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"))?;
    }
    issue_token_for_profile_locked(&state, claims.sub, equipment_profile_id).await
}
