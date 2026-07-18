use vespera::axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
};

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
