use sea_orm::entity::prelude::*;

/// 안전장비 종류 마스터 (FR-D1)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "equipment_types")]
pub struct Model {
    /// 장비 종류 고유번호
    #[sea_orm(primary_key)]
    pub equipment_type_id: i64,
    /// 장비명 (방폭형 안전화/내화학 장갑 등)
    pub name: String,
    /// 분류 (안전화/장갑/호흡보호구 등)
    pub category: Option<String>,
    /// 설명
    pub description: Option<String>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(has_many)]
    pub class_equipment_mappings: HasMany<super::class_equipment_mappings::Entity>,
    #[sea_orm(has_many)]
    pub ppe_requirements: HasMany<super::ppe_requirements::Entity>,
    #[sea_orm(has_many)]
    pub work_ppe_requirement_snapshots: HasMany<super::work_ppe_requirement_snapshots::Entity>,
    #[sea_orm(has_many)]
    pub equipment_profiles: HasMany<super::equipment_profiles::Entity>,
    #[sea_orm(has_many)]
    pub equipments: HasMany<super::equipment::Entity>,
}

vespera::schema_type!(Schema from Model, name = "EquipmentTypesSchema");
impl ActiveModelBehavior for ActiveModel {}
