use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sha2::{Digest, Sha256};
use vespera::axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
};

use crate::models::gate_terminals::{self, Entity as GateTerminals};
use crate::utils::{
    AppState,
    jwt::{self, Claims},
};

/// Authorization: Bearer 헤더에서 토큰을 꺼내 검증
fn claims_from_parts(parts: &Parts, state: &AppState) -> Result<Claims, StatusCode> {
    let header_value = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token = header_value
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    jwt::verify_token(token, &state.config.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)
}

/// 로그인한 사용자면 통과 (역할 무관)
/// 핸들러 파라미터에 `AuthUser(claims)`를 추가하면 해당 라우트가 보호됨
pub struct AuthUser(#[allow(dead_code)] pub Claims); // claims는 추후 감사로그 기록에 사용

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        claims_from_parts(parts, state).map(AuthUser)
    }
}

/// 관리 권한(ADMIN, SAFETY_MANAGER)만 통과
pub struct AdminUser(#[allow(dead_code)] pub Claims);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let claims = claims_from_parts(parts, state)?;
        if claims.role == "ADMIN" || claims.role == "SAFETY_MANAGER" {
            Ok(AdminUser(claims))
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// 단말 토큰 SHA-256 해시 (16진수 64자). 평문은 어디에도 저장하지 않는다.
pub fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

/// 게이트 단말 자격증명으로 통과 (FR-D5)
/// Authorization: Bearer <단말 토큰> — 해시로 조회하며 비활성·미등록 토큰은 401
pub struct GateTerminal(#[allow(dead_code)] pub gate_terminals::Model);

impl FromRequestParts<AppState> for GateTerminal {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let token = header_value
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let terminal = GateTerminals::find()
            .filter(gate_terminals::Column::TokenHash.eq(hash_token(token)))
            .one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if !terminal.is_active {
            return Err(StatusCode::UNAUTHORIZED);
        }
        Ok(GateTerminal(terminal))
    }
}
