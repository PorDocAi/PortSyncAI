use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "work_targets_work_target_type"
)]
pub enum WorkTargetType {
    #[sea_orm(string_value = "CONTAINER")]
    Container,
    #[sea_orm(string_value = "CARGO_ITEM")]
    CargoItem,
}

/// 작업별 컨테이너 또는 개별 화물 대상
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "work_targets")]
pub struct Model {
    /// 작업 대상 고유번호
    #[sea_orm(primary_key)]
    pub work_target_id: i64,
    /// 작업 FK
    pub work_id: i64,
    /// 대상 구분
    pub target_type: WorkTargetType,
    /// 컨테이너 대상 FK
    pub container_id: Option<i64>,
    /// 개별 화물 대상 FK
    pub cargo_item_id: Option<i64>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "work_id", to = "work_id")]
    pub work: HasOne<super::works::Entity>,
    #[sea_orm(belongs_to, from = "container_id", to = "container_id")]
    pub container: HasOne<super::containers::Entity>,
    #[sea_orm(belongs_to, from = "cargo_item_id", to = "cargo_item_id")]
    pub cargo_item: HasOne<super::cargo_items::Entity>,
}

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["work_id", "container_id"],  // uq_work_target_container
    &["work_id", "cargo_item_id"], // uq_work_target_cargo_item
];
vespera::schema_type!(Schema from Model, name = "WorkTargetsSchema");
impl ActiveModelBehavior for ActiveModel {}
