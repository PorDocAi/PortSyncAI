use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::models::cargo_documents::{Entity as CargoDocuments, ReviewStatus};
use crate::models::cargo_items::Entity as CargoItems;
use crate::models::work_assignments::{self, EligibilityStatus, Entity as WorkAssignments};
use crate::routes::attendances::today;
use crate::utils::{
    AppState,
    auth::AdminUser,
};

#[derive(Deserialize, vespera::Schema)]
pub struct CreateWorkAssignmentRequest {
    /// 작업자 직원 고유번호
    pub employee_id: i64,
    /// 배정 대상 화물 고유번호 (없으면 null — 공용 작업)
    pub cargo_item_id: Option<i64>,
}

#[derive(Serialize, vespera::Schema)]
pub struct WorkAssignmentResponse {
    pub assignment_id: i64,
    pub employee_id: i64,
    pub work_date: String,
    pub cargo_item_id: Option<i64>,
    pub assigned_by_id: i64,
    /// ELIGIBLE | EXCLUDED
    pub eligibility_status: String,
    pub created_at: String,
}

#[derive(Serialize, vespera::Schema)]
pub struct AssignmentErrorResponse {
    /// 기계 판독용 사유 코드 (예: MSDS_REVIEW_NOT_CONFIRMED)
    pub code: String,
    /// 작업자에게 표시할 한국어 사유
    pub message: String,
}

fn assignment_error(code: &str, message: &str) -> (StatusCode, Json<AssignmentErrorResponse>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(AssignmentErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
}

fn internal_error() -> (StatusCode, Json<AssignmentErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(AssignmentErrorResponse {
            code: "INTERNAL_ERROR".to_string(),
            message: "서버 오류가 발생했습니다.".to_string(),
        }),
    )
}

impl From<work_assignments::Model> for WorkAssignmentResponse {
    fn from(m: work_assignments::Model) -> Self {
        Self {
            assignment_id: m.assignment_id,
            employee_id: m.employee_id,
            work_date: m.work_date.to_string(),
            cargo_item_id: m.cargo_item_id,
            assigned_by_id: m.assigned_by_id,
            eligibility_status: match m.eligibility_status {
                EligibilityStatus::Eligible => "ELIGIBLE".to_string(),
                EligibilityStatus::Excluded => "EXCLUDED".to_string(),
            },
            created_at: m.created_at.to_rfc3339(),
        }
    }
}

/// 배정 대상 화물의 출처 문서(B/L·DGD 원문)가 검수 확정(CONFIRMED) 상태인지 확인한다 (시나리오 6).
/// 화물이 없으면 404, 연결된 문서가 없거나 검수 미완료면 422와 구조화된 사유를 반환한다.
async fn ensure_source_document_confirmed(
    db: &sea_orm::DatabaseConnection,
    cargo_item_id: i64,
) -> Result<(), (StatusCode, Json<AssignmentErrorResponse>)> {
    let item = CargoItems::find_by_id(cargo_item_id)
        .one(db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(AssignmentErrorResponse {
                    code: "CARGO_ITEM_NOT_FOUND".to_string(),
                    message: "배정 대상 화물을 찾을 수 없습니다.".to_string(),
                }),
            )
        })?;
    let Some(document_id) = item.cargo_document_id else {
        return Err(assignment_error(
            "MSDS_DOCUMENT_MISSING",
            "연결된 화물 문서가 없어 작업을 배정할 수 없습니다.",
        ));
    };
    let document = CargoDocuments::find_by_id(document_id)
        .one(db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| internal_error())?;
    if document.review_status != ReviewStatus::Confirmed {
        return Err(assignment_error(
            "MSDS_REVIEW_NOT_CONFIRMED",
            "MSDS 검수가 확정되지 않은 화물에는 작업을 배정할 수 없습니다.",
        ));
    }
    Ok(())
}

/// 작업 배정 생성 (FR-F3, 데모 시나리오 6 — 관리 권한)
/// 배정일은 요청 당일로 기록하고, 배정 담당자는 호출한 관리자 계정으로 남긴다.
#[vespera::route(post, tags = ["work_assignments"])]
pub async fn create_work_assignment(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateWorkAssignmentRequest>,
) -> Result<(StatusCode, Json<WorkAssignmentResponse>), (StatusCode, Json<AssignmentErrorResponse>)>
{
    if let Some(cargo_item_id) = req.cargo_item_id {
        ensure_source_document_confirmed(&state.db, cargo_item_id).await?;
    }

    let assignment = work_assignments::ActiveModel {
        employee_id: Set(req.employee_id),
        work_date: Set(today()),
        cargo_item_id: Set(req.cargo_item_id),
        assigned_by_id: Set(claims.sub),
        ..Default::default()
    };
    let saved = assignment
        .insert(&state.db)
        .await
        .map_err(|_| internal_error())?;

    Ok((StatusCode::CREATED, Json(WorkAssignmentResponse::from(saved))))
}

#[derive(Deserialize, vespera::Schema)]
pub struct UpdateWorkAssignmentRequest {
    /// 변경할 배정 대상 화물 고유번호 (생략 시 기존 값 유지)
    pub cargo_item_id: Option<i64>,
    /// ELIGIBLE | EXCLUDED (생략 시 기존 값 유지)
    pub eligibility_status: Option<EligibilityStatus>,
}

/// 작업 배정 수정 (관리 권한)
/// 배정 대상 화물을 변경하는 경우에도 검수 확정 여부를 동일하게 강제한다 (시나리오 6).
#[vespera::route(patch, path = "/{id}", tags = ["work_assignments"])]
pub async fn update_work_assignment(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateWorkAssignmentRequest>,
) -> Result<Json<WorkAssignmentResponse>, (StatusCode, Json<AssignmentErrorResponse>)> {
    let assignment = WorkAssignments::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(AssignmentErrorResponse {
                    code: "ASSIGNMENT_NOT_FOUND".to_string(),
                    message: "작업 배정을 찾을 수 없습니다.".to_string(),
                }),
            )
        })?;

    if let Some(cargo_item_id) = req.cargo_item_id {
        ensure_source_document_confirmed(&state.db, cargo_item_id).await?;
    }

    let mut active: work_assignments::ActiveModel = assignment.into();
    if let Some(cargo_item_id) = req.cargo_item_id {
        active.cargo_item_id = Set(Some(cargo_item_id));
    }
    if let Some(eligibility_status) = req.eligibility_status {
        active.eligibility_status = Set(eligibility_status);
    }
    active.updated_at = Set(Some(chrono::Utc::now().into()));
    let saved = active.update(&state.db).await.map_err(|_| internal_error())?;

    Ok(Json(WorkAssignmentResponse::from(saved)))
}
