use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::models::employees::{self, Entity as Employees, SystemRole};
use crate::routes::auth::role_as_str;
use crate::utils::{
    AppState,
    auth::{AdminUser, AuthUser},
};

/// 응답 DTO — password_hash를 절대 밖으로 내보내지 않기 위해 Model과 분리
#[derive(Serialize, vespera::Schema)]
pub struct EmployeeResponse {
    pub employee_id: i64,
    pub employee_number: String,
    pub name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub department_id: i64,
    pub job_role_id: i64,
    pub position: Option<String>,
    pub system_role: String,
    pub preferred_language: String,
    pub hire_date: String,
    pub status: String,
}

impl From<employees::Model> for EmployeeResponse {
    fn from(m: employees::Model) -> Self {
        Self {
            employee_id: m.employee_id,
            employee_number: m.employee_number,
            name: m.name,
            email: m.email,
            phone_number: m.phone_number,
            department_id: m.department_id,
            job_role_id: m.job_role_id,
            position: m.position,
            system_role: role_as_str(&m.system_role).to_string(),
            preferred_language: m.preferred_language,
            hire_date: m.hire_date.to_string(),
            status: format!("{:?}", m.status).to_uppercase(),
        }
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct CreateEmployeeRequest {
    pub employee_number: String,
    pub name: String,
    pub email: String,
    pub password: String,
    pub phone_number: Option<String>,
    pub department_id: i64,
    pub job_role_id: i64,
    pub position: Option<String>,
    pub system_role: Option<SystemRole>,
    pub preferred_language: Option<String>,
    /// YYYY-MM-DD
    pub hire_date: String,
}

/// 직원 목록 조회
#[vespera::route(get, tags = ["employees"])]
pub async fn list_employees(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<EmployeeResponse>>, StatusCode> {
    let rows = Employees::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(EmployeeResponse::from).collect()))
}

/// 직원 단건 조회
#[vespera::route(get, path = "/{id}", tags = ["employees"])]
pub async fn get_employee(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<EmployeeResponse>, StatusCode> {
    let row = Employees::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(EmployeeResponse::from(row)))
}

/// 직원 등록 (비밀번호는 argon2 해시 저장)
#[vespera::route(post, tags = ["employees"])]
pub async fn create_employee(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateEmployeeRequest>,
) -> Result<(StatusCode, Json<EmployeeResponse>), StatusCode> {
    let hire_date = req
        .hire_date
        .parse::<chrono::NaiveDate>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let new_employee = employees::ActiveModel {
        employee_number: Set(req.employee_number),
        name: Set(req.name),
        email: Set(req.email),
        password_hash: Set(password_hash),
        phone_number: Set(req.phone_number),
        department_id: Set(req.department_id),
        job_role_id: Set(req.job_role_id),
        position: Set(req.position),
        system_role: Set(req.system_role.unwrap_or(SystemRole::Worker)),
        preferred_language: Set(req.preferred_language.unwrap_or_else(|| "ko".to_string())),
        hire_date: Set(hire_date),
        ..Default::default()
    };

    let saved = new_employee
        .insert(&state.db)
        .await
        // 사번/이메일 unique 충돌 포함 — 상세 분기는 추후
        .map_err(|_| StatusCode::CONFLICT)?;

    Ok((StatusCode::CREATED, Json(EmployeeResponse::from(saved))))
}
