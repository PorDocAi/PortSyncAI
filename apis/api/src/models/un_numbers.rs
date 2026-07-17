use sea_orm::entity::prelude::*;

/// UN 번호 마스터 (FR-B1, UN No → Class 확정)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "un_numbers")]
pub struct Model {
    /// UN번호 고유번호
    #[sea_orm(primary_key)]
    pub un_number_id: i64,
    /// UN No. (4자리)
    #[sea_orm(unique)]
    pub un_number: String,
    /// 위험물 등급 FK
    #[sea_orm(indexed)]
    pub dg_class_id: i64,
    /// 정식 운송품명
    pub proper_shipping_name: Option<String>,
    /// 포장등급 (I/II/III)
    pub packing_group: Option<String>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "dg_class_id", to = "dg_class_id")]
    pub dg_class: HasOne<super::dg_classes::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [dg_class_id]
vespera::schema_type!(Schema from Model, name = "UnNumbersSchema");
impl ActiveModelBehavior for ActiveModel {}
