use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::models::job_roles::{self, Entity as JobRoles};
use crate::utils::{
    AppState,
    auth::{AdminUser, AuthUser},
};

#[derive(Serialize, vespera::Schema)]
pub struct JobRoleResponse {
    pub job_role_id: i64,
    pub job_role_code: String,
    pub name: String,
    pub description: Option<String>,
}

impl From<job_roles::Model> for JobRoleResponse {
    fn from(m: job_roles::Model) -> Self {
        Self {
            job_role_id: m.job_role_id,
            job_role_code: m.job_role_code,
            name: m.name,
            description: m.description,
        }
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct CreateJobRoleRequest {
    pub job_role_code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, vespera::Schema)]
pub struct UpdateJobRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// 직무 목록 조회
#[vespera::route(get, tags = ["job_roles"])]
pub async fn list_job_roles(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<JobRoleResponse>>, StatusCode> {
    let rows = JobRoles::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(JobRoleResponse::from).collect()))
}

/// 직무 생성 (관리 권한)
#[vespera::route(post, tags = ["job_roles"])]
pub async fn create_job_role(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateJobRoleRequest>,
) -> Result<(StatusCode, Json<JobRoleResponse>), StatusCode> {
    let new_job_role = job_roles::ActiveModel {
        job_role_code: Set(req.job_role_code),
        name: Set(req.name),
        description: Set(req.description),
        ..Default::default()
    };
    let saved = new_job_role
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::CONFLICT)?;
    Ok((StatusCode::CREATED, Json(JobRoleResponse::from(saved))))
}

/// 직무 수정 (관리 권한)
#[vespera::route(put, path = "/{id}", tags = ["job_roles"])]
pub async fn update_job_role(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateJobRoleRequest>,
) -> Result<Json<JobRoleResponse>, StatusCode> {
    let row = JobRoles::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active: job_roles::ActiveModel = row.into();
    if let Some(name) = req.name {
        active.name = Set(name);
    }
    if let Some(description) = req.description {
        active.description = Set(Some(description));
    }
    active.updated_at = Set(Some(chrono::Utc::now().into()));

    let saved = active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(JobRoleResponse::from(saved)))
}
