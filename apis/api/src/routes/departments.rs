use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::models::departments::{self, Entity as Departments};
use crate::utils::{
    AppState,
    auth::{AdminUser, AuthUser},
};

#[derive(Serialize, vespera::Schema)]
pub struct DepartmentResponse {
    pub department_id: i64,
    pub department_code: String,
    pub name: String,
    pub description: Option<String>,
}

impl From<departments::Model> for DepartmentResponse {
    fn from(m: departments::Model) -> Self {
        Self {
            department_id: m.department_id,
            department_code: m.department_code,
            name: m.name,
            description: m.description,
        }
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct CreateDepartmentRequest {
    pub department_code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, vespera::Schema)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// 부서 목록 조회
#[vespera::route(get, tags = ["departments"])]
pub async fn list_departments(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<DepartmentResponse>>, StatusCode> {
    let rows = Departments::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        rows.into_iter().map(DepartmentResponse::from).collect(),
    ))
}

/// 부서 생성 (관리 권한)
#[vespera::route(post, tags = ["departments"])]
pub async fn create_department(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<(StatusCode, Json<DepartmentResponse>), StatusCode> {
    let new_department = departments::ActiveModel {
        department_code: Set(req.department_code),
        name: Set(req.name),
        description: Set(req.description),
        ..Default::default()
    };
    let saved = new_department
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::CONFLICT)?;
    Ok((StatusCode::CREATED, Json(DepartmentResponse::from(saved))))
}

/// 부서 수정 (관리 권한)
#[vespera::route(put, path = "/{id}", tags = ["departments"])]
pub async fn update_department(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> Result<Json<DepartmentResponse>, StatusCode> {
    let row = Departments::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active: departments::ActiveModel = row.into();
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
    Ok(Json(DepartmentResponse::from(saved)))
}
