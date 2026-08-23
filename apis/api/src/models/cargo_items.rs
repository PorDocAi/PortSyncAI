use sea_orm::entity::prelude::*;

/// 입고 화물 (B/L·DGD에서 추출·확정, FR-B1~B4)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cargo_items")]
pub struct Model {
    /// 화물 고유번호
    #[sea_orm(primary_key)]
    pub cargo_item_id: i64,
    /// 출처 화물문서 FK
    pub cargo_document_id: Option<i64>,
    /// 선하증권 번호
    pub bl_number: Option<String>,
    /// 위험물신고서 번호 (부재 시 null)
    pub dgd_number: Option<String>,
    /// 추출된 UN No.
    pub un_number: Option<String>,
    /// 확정된 위험물 등급 FK
    #[sea_orm(indexed)]
    pub dg_class_id: Option<i64>,
    /// B/L HS Code
    pub hs_code: Option<String>,
    /// 품목명 (B/L 16번 Description)
    pub item_name: Option<String>,
    /// 화물 상세
    pub description: Option<String>,
    /// 위험물 여부
    #[sea_orm(default_value = false)]
    pub is_dangerous: bool,
    /// DGD 누락 경고 (FR-B3)
    #[sea_orm(default_value = false)]
    pub dgd_missing_warning: bool,
    /// 입고 예정일 (FR-C1 당일 매칭)
    #[sea_orm(indexed)]
    pub arrival_date: Option<Date>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "cargo_document_id", to = "cargo_document_id")]
    pub cargo_document: HasOne<super::cargo_documents::Entity>,
    #[sea_orm(belongs_to, from = "dg_class_id", to = "dg_class_id")]
    pub dg_class: HasOne<super::dg_classes::Entity>,
    #[sea_orm(has_many)]
    pub cargo_item_documents: HasMany<super::cargo_item_documents::Entity>,
    #[sea_orm(has_many)]
    pub work_assignments: HasMany<super::work_assignments::Entity>,
    #[sea_orm(has_many)]
    pub container_cargo_items: HasMany<super::container_cargo_items::Entity>,
    #[sea_orm(has_many)]
    pub work_targets: HasMany<super::work_targets::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [dg_class_id]
// (unnamed) on [arrival_date]
vespera::schema_type!(Schema from Model, name = "CargoItemsSchema");
impl ActiveModelBehavior for ActiveModel {}
