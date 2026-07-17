use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

/// JWT에 담기는 정보 (스프링의 UserDetails에 해당하는 최소 정보)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 직원 고유번호 (employees.employee_id)
    pub sub: i64,
    /// 시스템 권한 (ADMIN / SAFETY_MANAGER / SUPERVISOR / WORKER)
    pub role: String,
    /// 만료 시각 (unix timestamp)
    pub exp: i64,
}

const TOKEN_TTL_HOURS: i64 = 12;

pub fn create_token(
    employee_id: i64,
    role: &str,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(TOKEN_TTL_HOURS)).timestamp();
    let claims = Claims {
        sub: employee_id,
        role: role.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

#[allow(dead_code)]
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
