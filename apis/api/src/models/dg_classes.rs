use sea_orm::entity::prelude::*;

/// 위험물 등급 (IMDG Class) 마스터 (FR-B, 설정 데이터)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "dg_classes")]
pub struct Model {
    /// 등급 고유번호
    #[sea_orm(primary_key)]
    pub dg_class_id: i64,
    /// Class 코드 (예: 1, 2.1, 3, 9)
    #[sea_orm(unique)]
    pub class_code: String,
    /// 등급 한글명 (화약류/가스류 등)
    pub name_ko: String,
    /// 등급 영문명
    pub name_en: Option<String>,
    /// 설명
    pub description: Option<String>,
    /// 적용 IMDG 개정 차수 (예: 42차)
    pub imdg_version: Option<String>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(has_many)]
    pub class_equipment_mappings: HasMany<super::class_equipment_mappings::Entity>,
    #[sea_orm(has_many)]
    pub class_instruction_mappings: HasMany<super::class_instruction_mappings::Entity>,
    #[sea_orm(has_many)]
    pub cargo_items: HasMany<super::cargo_items::Entity>,
    #[sea_orm(has_many)]
    pub work_restriction_rules: HasMany<super::work_restriction_rules::Entity>,
    #[sea_orm(has_many)]
    pub hs_codes: HasMany<super::hs_codes::Entity>,
    #[sea_orm(has_many)]
    pub un_numbers: HasMany<super::un_numbers::Entity>,
    #[sea_orm(has_many)]
    pub education_target_rules: HasMany<super::education_target_rules::Entity>,
}

vespera::schema_type!(Schema from Model, name = "DgClassesSchema");
impl ActiveModelBehavior for ActiveModel {}
