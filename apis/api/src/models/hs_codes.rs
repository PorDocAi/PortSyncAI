use sea_orm::entity::prelude::*;

/// HS Code 마스터 (FR-B2/B3, 위험물 의심 판정)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "hs_codes")]
pub struct Model {
    /// HS Code 고유번호
    #[sea_orm(primary_key)]
    pub hs_code_id: i64,
    /// HS Code
    #[sea_orm(unique)]
    pub hs_code: String,
    /// 품목 설명
    pub description: Option<String>,
    /// 위험물 의심 여부 (DGD 누락 검증용, FR-B3)
    #[sea_orm(default_value = false)]
    pub is_dangerous_suspect: bool,
    /// 기본 매핑 등급 FK
    pub default_dg_class_id: Option<i64>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "default_dg_class_id", to = "dg_class_id")]
    pub default: HasOne<super::dg_classes::Entity>,
}

vespera::schema_type!(Schema from Model, name = "HsCodesSchema");
impl ActiveModelBehavior for ActiveModel {}
