use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::models::v2_work_assignments::Entity as V2WorkAssignments;
use crate::models::work_stops::{self, Entity as WorkStops, WorkStopStatus};
use crate::models::works::Entity as Works;
use crate::utils::{
    AppState,
    auth::{AdminUser, AuthUser},
};

#[derive(Serialize, vespera::Schema)]
pub struct WorkStopErrorResponse {
    pub code: String,
    pub message: String,
}

fn stop_error(
    status: StatusCode,
    code: &str,
    message: &str,
) -> (StatusCode, Json<WorkStopErrorResponse>) {
    (
        status,
        Json(WorkStopErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
}

fn internal_error() -> (StatusCode, Json<WorkStopErrorResponse>) {
    stop_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "서버 오류가 발생했습니다.",
    )
}

fn status_as_str(status: &WorkStopStatus) -> &'static str {
    match status {
        WorkStopStatus::Open => "OPEN",
        WorkStopStatus::Closed => "CLOSED",
    }
}

#[derive(Serialize, vespera::Schema)]
pub struct WorkStopResponse {
    pub work_stop_id: i64,
    pub work_id: Option<i64>,
    pub work_assignment_id: Option<i64>,
    pub status: String,
    pub reason: String,
    pub stopped_by_id: Option<i64>,
    pub stopped_at: String,
    pub closed_by_id: Option<i64>,
    pub closed_at: Option<String>,
    /// CLOSED 이면 충족(해제됨), OPEN 이면 미충족(작업 차단)
    pub fulfilled: bool,
}

impl From<work_stops::Model> for WorkStopResponse {
    fn from(m: work_stops::Model) -> Self {
        let fulfilled = m.status == WorkStopStatus::Closed;
        Self {
            work_stop_id: m.work_stop_id,
            work_id: m.work_id,
            work_assignment_id: m.work_assignment_id,
            status: status_as_str(&m.status).to_string(),
            reason: m.reason,
            stopped_by_id: m.stopped_by_id,
            stopped_at: m.stopped_at.to_rfc3339(),
            closed_by_id: m.closed_by_id,
            closed_at: m.closed_at.map(|ts| ts.to_rfc3339()),
            fulfilled,
        }
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct CreateWorkStopRequest {
    pub work_id: Option<i64>,
    pub work_assignment_id: Option<i64>,
    pub reason: String,
}

#[derive(Deserialize, vespera::Schema)]
pub struct WorkStopListQuery {
    pub work_id: Option<i64>,
    pub work_assignment_id: Option<i64>,
    /// OPEN | CLOSED
    pub status: Option<String>,
}

/// 작업중지 목록. OPEN 은 미충족, CLOSED 는 충족으로 구분한다.
#[vespera::route(get, tags = ["work_stops"])]
pub async fn list_work_stops(
    _auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<WorkStopListQuery>,
) -> Result<Json<Vec<WorkStopResponse>>, (StatusCode, Json<WorkStopErrorResponse>)> {
    let mut query = WorkStops::find();
    if let Some(work_id) = q.work_id {
        query = query.filter(work_stops::Column::WorkId.eq(work_id));
    }
    if let Some(work_assignment_id) = q.work_assignment_id {
        query = query.filter(work_stops::Column::WorkAssignmentId.eq(work_assignment_id));
    }
    if let Some(status) = q.status {
        let status = match status.as_str() {
            "OPEN" => WorkStopStatus::Open,
            "CLOSED" => WorkStopStatus::Closed,
            _ => {
                return Err(stop_error(
                    StatusCode::BAD_REQUEST,
                    "INVALID_WORK_STOP_STATUS",
                    "status 는 OPEN 또는 CLOSED 여야 합니다.",
                ));
            }
        };
        query = query.filter(work_stops::Column::Status.eq(status));
    }
    let rows = query
        .order_by_desc(work_stops::Column::WorkStopId)
        .all(&state.db)
        .await
        .map_err(|_| internal_error())?;
    Ok(Json(rows.into_iter().map(WorkStopResponse::from).collect()))
}

/// 작업 또는 v2 배정 범위의 작업중지를 연다 (관리 권한)
#[vespera::route(post, tags = ["work_stops"])]
pub async fn create_work_stop(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateWorkStopRequest>,
) -> Result<(StatusCode, Json<WorkStopResponse>), (StatusCode, Json<WorkStopErrorResponse>)> {
    if req.reason.trim().is_empty() {
        return Err(stop_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "WORK_STOP_REASON_REQUIRED",
            "작업중지 사유가 필요합니다.",
        ));
    }

    let mut work_id = req.work_id;
    let mut work_assignment_id = req.work_assignment_id;
    if work_id.is_none() && work_assignment_id.is_none() {
        return Err(stop_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "WORK_STOP_TARGET_REQUIRED",
            "작업 또는 v2 작업 배정 중 하나 이상이 필요합니다.",
        ));
    }

    if let Some(assignment_id) = work_assignment_id {
        let assignment = V2WorkAssignments::find_by_id(assignment_id)
            .one(&state.db)
            .await
            .map_err(|_| internal_error())?
            .ok_or_else(|| {
                stop_error(
                    StatusCode::NOT_FOUND,
                    "WORK_ASSIGNMENT_NOT_FOUND",
                    "v2 작업 배정을 찾을 수 없습니다.",
                )
            })?;
        match work_id {
            Some(id) if id != assignment.work_id => {
                return Err(stop_error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "WORK_STOP_TARGET_MISMATCH",
                    "작업과 v2 작업 배정이 일치하지 않습니다.",
                ));
            }
            None => work_id = Some(assignment.work_id),
            Some(_) => {}
        }
        work_assignment_id = Some(assignment.work_assignment_id);
    }

    if let Some(id) = work_id {
        Works::find_by_id(id)
            .one(&state.db)
            .await
            .map_err(|_| internal_error())?
            .ok_or_else(|| {
                stop_error(
                    StatusCode::NOT_FOUND,
                    "WORK_NOT_FOUND",
                    "작업을 찾을 수 없습니다.",
                )
            })?;
    }

    let stop = work_stops::ActiveModel {
        work_id: Set(work_id),
        work_assignment_id: Set(work_assignment_id),
        status: Set(WorkStopStatus::Open),
        reason: Set(req.reason),
        stopped_by_id: Set(Some(claims.sub)),
        ..Default::default()
    };
    let saved = stop.insert(&state.db).await.map_err(|_| internal_error())?;
    Ok((StatusCode::CREATED, Json(WorkStopResponse::from(saved))))
}

/// 작업중지를 해제한다 (관리 권한). 이미 CLOSED 이면 멱등하게 200을 반환한다.
#[vespera::route(post, path = "/{id}/close", tags = ["work_stops"])]
pub async fn close_work_stop(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<WorkStopResponse>, (StatusCode, Json<WorkStopErrorResponse>)> {
    let stop = WorkStops::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| {
            stop_error(
                StatusCode::NOT_FOUND,
                "WORK_STOP_NOT_FOUND",
                "작업중지를 찾을 수 없습니다.",
            )
        })?;
    if stop.status == WorkStopStatus::Closed {
        return Ok(Json(WorkStopResponse::from(stop)));
    }

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let mut active: work_stops::ActiveModel = stop.into();
    active.status = Set(WorkStopStatus::Closed);
    active.closed_by_id = Set(Some(claims.sub));
    active.closed_at = Set(Some(now));
    active.updated_at = Set(Some(now));
    let saved = active
        .update(&state.db)
        .await
        .map_err(|_| internal_error())?;
    Ok(Json(WorkStopResponse::from(saved)))
}

pub async fn has_open_work_stop(
    db: &sea_orm::DatabaseConnection,
    work_id: i64,
    work_assignment_id: i64,
) -> Result<bool, StatusCode> {
    let open = WorkStops::find()
        .filter(work_stops::Column::Status.eq(WorkStopStatus::Open))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(open.iter().any(|stop| {
        stop.work_assignment_id == Some(work_assignment_id) || stop.work_id == Some(work_id)
    }))
}
