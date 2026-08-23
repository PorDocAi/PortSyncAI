use sea_orm::entity::prelude::*;

/// 작업 대상 컨테이너
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "containers")]
pub struct Model {
    /// 컨테이너 고유번호
    #[sea_orm(primary_key)]
    pub container_id: i64,
    /// ISO 6346 컨테이너 번호
    #[sea_orm(unique)]
    pub container_number: String,
    /// 봉인 번호
    pub seal_number: Option<String>,
    /// 터미널 도착일시
    pub arrival_at: Option<DateTimeWithTimeZone>,
    /// 터미널 출발일시
    pub departure_at: Option<DateTimeWithTimeZone>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(has_many)]
    pub container_cargo_items: HasMany<super::container_cargo_items::Entity>,
    #[sea_orm(has_many)]
    pub work_targets: HasMany<super::work_targets::Entity>,
}

vespera::schema_type!(Schema from Model, name = "ContainersSchema");
impl ActiveModelBehavior for ActiveModel {}
