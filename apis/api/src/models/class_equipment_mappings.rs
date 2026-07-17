use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "class_equipment_mappings_requirement_level"
)]
pub enum RequirementLevel {
    #[sea_orm(string_value = "REQUIRED")]
    Required,
    #[sea_orm(string_value = "RECOMMENDED")]
    Recommended,
}

/// 위험물 등급 ↔ 필수/권장 안전장비 매핑 (FR-B5, D1, 설정 데이터)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "class_equipment_mappings")]
pub struct Model {
    /// 매핑 고유번호
    #[sea_orm(primary_key)]
    pub mapping_id: i64,
    /// 위험물 등급 FK
    pub dg_class_id: i64,
    /// 장비 종류 FK
    pub equipment_type_id: i64,
    /// 필수/권장 구분
    #[sea_orm(default_value = "REQUIRED")]
    pub requirement_level: RequirementLevel,
    /// 적용 IMDG 차수
    pub imdg_version: Option<String>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "dg_class_id", to = "dg_class_id")]
    pub dg_class: HasOne<super::dg_classes::Entity>,
    #[sea_orm(belongs_to, from = "equipment_type_id", to = "equipment_type_id")]
    pub equipment_type: HasOne<super::equipment_types::Entity>,
}

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["dg_class_id", "equipment_type_id"], // uq_class_equipment
];
vespera::schema_type!(Schema from Model, name = "ClassEquipmentMappingsSchema");
impl ActiveModelBehavior for ActiveModel {}
