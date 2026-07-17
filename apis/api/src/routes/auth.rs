use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use vespera::axum::{Json, extract::State, http::StatusCode};

use crate::models::employees::{self, EmployeeStatus, Entity as Employees, SystemRole};
use crate::utils::{AppState, jwt};

pub fn role_as_str(role: &SystemRole) -> &'static str {
    match role {
        SystemRole::Admin => "ADMIN",
        SystemRole::SafetyManager => "SAFETY_MANAGER",
        SystemRole::Supervisor => "SUPERVISOR",
        SystemRole::Worker => "WORKER",
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, vespera::Schema)]
pub struct SignInResponse {
    pub token: String,
    pub employee_id: i64,
    pub name: String,
    pub system_role: String,
}

/// 로그인: 이메일+비밀번호 검증 후 JWT 발급
#[vespera::route(post, path = "/signin", tags = ["auth"])]
pub async fn signin(
    State(state): State<AppState>,
    Json(req): Json<SignInRequest>,
) -> Result<Json<SignInResponse>, StatusCode> {
    let employee = Employees::find()
        .filter(employees::Column::Email.eq(&req.email))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        // 계정이 없어도 비밀번호 오류와 같은 401 (계정 존재 여부 노출 방지)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if employee.status != EmployeeStatus::Active {
        return Err(StatusCode::FORBIDDEN);
    }

    let parsed_hash = PasswordHash::new(&employee.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let role = role_as_str(&employee.system_role).to_string();
    let token = jwt::create_token(employee.employee_id, &role, &state.config.jwt_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SignInResponse {
        token,
        employee_id: employee.employee_id,
        name: employee.name,
        system_role: role,
    }))
}
