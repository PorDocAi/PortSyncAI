use sea_orm::entity::prelude::*;

/// 위험물 등급 ↔ 안전지침 매핑 (FR-B4, 화물유형별 지침 자동 전송)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "class_instruction_mappings")]
pub struct Model {
    /// 매핑 고유번호
    #[sea_orm(primary_key)]
    pub mapping_id: i64,
    /// 위험물 등급 FK
    pub dg_class_id: i64,
    /// 안전지침 FK
    pub instruction_id: i64,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "dg_class_id", to = "dg_class_id")]
    pub dg_class: HasOne<super::dg_classes::Entity>,
    #[sea_orm(belongs_to, from = "instruction_id", to = "instruction_id")]
    pub instruction: HasOne<super::safety_instructions::Entity>,
}

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["dg_class_id", "instruction_id"], // uq_class_instruction
];
vespera::schema_type!(Schema from Model, name = "ClassInstructionMappingsSchema");
impl ActiveModelBehavior for ActiveModel {}
