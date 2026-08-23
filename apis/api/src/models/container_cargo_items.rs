use sea_orm::entity::prelude::*;

/// 컨테이너와 혼재 화물의 다대다 관계
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "container_cargo_items")]
pub struct Model {
    /// 컨테이너 화물 관계 고유번호
    #[sea_orm(primary_key)]
    pub container_cargo_item_id: i64,
    /// 컨테이너 FK
    pub container_id: i64,
    /// 화물 FK
    pub cargo_item_id: i64,
    /// 적재 확인일시
    pub loaded_at: Option<DateTimeWithTimeZone>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "container_id", to = "container_id")]
    pub container: HasOne<super::containers::Entity>,
    #[sea_orm(belongs_to, from = "cargo_item_id", to = "cargo_item_id")]
    pub cargo_item: HasOne<super::cargo_items::Entity>,
}


/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["container_id", "cargo_item_id"], // uq_container_cargo_item
];
vespera::schema_type!(Schema from Model, name = "ContainerCargoItemsSchema");
impl ActiveModelBehavior for ActiveModel {}
