use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::models::v2_work_assignments::Entity as V2WorkAssignments;
use crate::models::work_preparations::{self, Entity as WorkPreparations, WorkPreparationStatus};
use crate::routes::education::evaluate_education_readiness;
use crate::routes::ppe::snapshot_ids_for_work;
use crate::routes::work_stops::has_open_work_stop;
use crate::utils::{
    AppState,
    auth::{AdminUser, AuthUser},
};

#[derive(Serialize, vespera::Schema)]
pub struct WorkPreparationErrorResponse {
    pub code: String,
    pub message: String,
}

fn prep_error(
    status: StatusCode,
    code: &str,
    message: &str,
) -> (StatusCode, Json<WorkPreparationErrorResponse>) {
    (
        status,
        Json(WorkPreparationErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
}

fn internal_error() -> (StatusCode, Json<WorkPreparationErrorResponse>) {
    prep_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "서버 오류가 발생했습니다.",
    )
}

fn status_as_str(status: &WorkPreparationStatus) -> &'static str {
    match status {
        WorkPreparationStatus::NotStarted => "NOT_STARTED",
        WorkPreparationStatus::InProgress => "IN_PROGRESS",
        WorkPreparationStatus::Ready => "READY",
        WorkPreparationStatus::Blocked => "BLOCKED",
    }
}

#[derive(Serialize, vespera::Schema)]
pub struct WorkPreparationResponse {
    pub work_preparation_id: i64,
    pub work_assignment_id: i64,
    pub status: String,
    pub document_ready: bool,
    pub education_ready: bool,
    pub instruction_ready: bool,
    pub ppe_ready: bool,
    pub decision_input_versions: serde_json::Value,
    pub prepared_at: Option<String>,
    /// 문서·지침·교육·PPE가 충족되고 열린 작업중지가 없으면 true
    pub fulfilled: bool,
}

impl From<work_preparations::Model> for WorkPreparationResponse {
    fn from(m: work_preparations::Model) -> Self {
        let fulfilled = m.document_ready
            && m.education_ready
            && m.instruction_ready
            && m.ppe_ready
            && m.status == WorkPreparationStatus::Ready;
        Self {
            work_preparation_id: m.work_preparation_id,
            work_assignment_id: m.work_assignment_id,
            status: status_as_str(&m.status).to_string(),
            document_ready: m.document_ready,
            education_ready: m.education_ready,
            instruction_ready: m.instruction_ready,
            ppe_ready: m.ppe_ready,
            decision_input_versions: m.decision_input_versions,
            prepared_at: m.prepared_at.map(|ts| ts.to_rfc3339()),
            fulfilled,
        }
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct UpsertWorkPreparationRequest {
    pub work_assignment_id: i64,
    pub document_ready: Option<bool>,
    pub instruction_ready: Option<bool>,
}

/// v2 배정 준비 상태를 생성하거나 재계산한다 (관리 권한)
#[vespera::route(post, tags = ["work_preparations"])]
pub async fn upsert_work_preparation(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<UpsertWorkPreparationRequest>,
) -> Result<
    (StatusCode, Json<WorkPreparationResponse>),
    (StatusCode, Json<WorkPreparationErrorResponse>),
> {
    let saved = refresh_work_preparation(
        &state.db,
        req.work_assignment_id,
        req.document_ready,
        req.instruction_ready,
    )
    .await?;
    let created = saved.updated_at.is_none();
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(WorkPreparationResponse::from(saved))))
}

/// v2 배정 준비 상태 조회
#[vespera::route(get, path = "/{id}", tags = ["work_preparations"])]
pub async fn get_work_preparation(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<WorkPreparationResponse>, (StatusCode, Json<WorkPreparationErrorResponse>)> {
    let row = WorkPreparations::find()
        .filter(work_preparations::Column::WorkAssignmentId.eq(id))
        .one(&state.db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| {
            prep_error(
                StatusCode::NOT_FOUND,
                "WORK_PREPARATION_NOT_FOUND",
                "작업 준비 상태를 찾을 수 없습니다.",
            )
        })?;
    Ok(Json(WorkPreparationResponse::from(row)))
}

async fn refresh_work_preparation(
    db: &sea_orm::DatabaseConnection,
    work_assignment_id: i64,
    document_ready: Option<bool>,
    instruction_ready: Option<bool>,
) -> Result<work_preparations::Model, (StatusCode, Json<WorkPreparationErrorResponse>)> {
    let assignment = V2WorkAssignments::find_by_id(work_assignment_id)
        .one(db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| {
            prep_error(
                StatusCode::NOT_FOUND,
                "WORK_ASSIGNMENT_NOT_FOUND",
                "v2 작업 배정을 찾을 수 없습니다.",
            )
        })?;

    let education = evaluate_education_readiness(db, assignment.employee_id, assignment.work_id)
        .await
        .map_err(|(status, Json(err))| prep_error(status, &err.code, &err.message))?;
    let snapshot_ids = snapshot_ids_for_work(db, assignment.work_id)
        .await
        .map_err(|_| internal_error())?;
    let open_stop = has_open_work_stop(db, assignment.work_id, assignment.work_assignment_id)
        .await
        .map_err(|_| internal_error())?;

    let existing = WorkPreparations::find()
        .filter(work_preparations::Column::WorkAssignmentId.eq(work_assignment_id))
        .one(db)
        .await
        .map_err(|_| internal_error())?;

    let education_ready = education.fulfilled;
    // Snapshots are the authoritative PPE requirements for a work. An empty
    // snapshot set means this work has no PPE requirement to satisfy.
    let ppe_ready = true;
    let document_ready = document_ready.unwrap_or_else(|| {
        existing
            .as_ref()
            .map(|row| row.document_ready)
            .unwrap_or(false)
    });
    let instruction_ready = instruction_ready.unwrap_or_else(|| {
        existing
            .as_ref()
            .map(|row| row.instruction_ready)
            .unwrap_or(false)
    });
    let fulfilled = education_ready && ppe_ready && !open_stop;
    let status = if open_stop || !education_ready {
        WorkPreparationStatus::Blocked
    } else if fulfilled && document_ready && instruction_ready {
        WorkPreparationStatus::Ready
    } else if education_ready || ppe_ready || document_ready || instruction_ready {
        WorkPreparationStatus::InProgress
    } else {
        WorkPreparationStatus::NotStarted
    };
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let prepared_at = if status == WorkPreparationStatus::Ready {
        Some(now)
    } else {
        None
    };
    let decision_input_versions = serde_json::json!({
        "education_fulfilled": education_ready,
        "education_requirement_statuses": education.requirements.iter().map(|item| {
            serde_json::json!({
                "education_course_id": item.education_course_id,
                "status": item.status,
            })
        }).collect::<Vec<_>>(),
        "ppe_snapshot_ids": snapshot_ids,
        "open_work_stop": open_stop,
    });

    if let Some(existing) = existing {
        let mut active: work_preparations::ActiveModel = existing.into();
        active.status = Set(status);
        active.document_ready = Set(document_ready);
        active.education_ready = Set(education_ready);
        active.instruction_ready = Set(instruction_ready);
        active.ppe_ready = Set(ppe_ready);
        active.decision_input_versions = Set(decision_input_versions);
        active.prepared_at = Set(prepared_at);
        active.updated_at = Set(Some(now));
        return active.update(db).await.map_err(|error| {
            eprintln!("work preparation update database error: {error:?}");
            internal_error()
        });
    }

    let created = work_preparations::ActiveModel {
        work_assignment_id: Set(work_assignment_id),
        status: Set(status),
        document_ready: Set(document_ready),
        education_ready: Set(education_ready),
        instruction_ready: Set(instruction_ready),
        ppe_ready: Set(ppe_ready),
        decision_input_versions: Set(decision_input_versions),
        prepared_at: Set(prepared_at),
        ..Default::default()
    };
    created.insert(db).await.map_err(|error| {
        eprintln!("work preparation create database error: {error:?}");
        internal_error()
    })
}
