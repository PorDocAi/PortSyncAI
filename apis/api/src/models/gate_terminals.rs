use sea_orm::entity::prelude::*;

/// 게이트 단말 자격증명 (FR-D5 — 게이트 리더기 전용 인증)
/// 발급 시 평문 토큰은 응답으로 한 번만 반환하고 SHA-256 해시만 저장한다.
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "gate_terminals")]
pub struct Model {
    /// 단말 고유번호
    #[sea_orm(primary_key)]
    pub terminal_id: i64,
    /// 게이트 식별자 (예: 북문, 남문)
    pub gate_id: String,
    /// 단말 토큰 SHA-256 해시 (평문 저장 금지)
    #[sea_orm(unique)]
    pub token_hash: String,
    /// 활성 여부 (폐기 시 false — 소프트 삭제)
    #[sea_orm(default_value = true)]
    pub is_active: bool,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
}

vespera::schema_type!(Schema from Model, name = "GateTerminalsSchema");
impl ActiveModelBehavior for ActiveModel {}
