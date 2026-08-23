use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "cargo_item_documents_cargo_document_role")]
pub enum CargoDocumentRole {
    #[sea_orm(string_value = "BL")]
    Bl,
    #[sea_orm(string_value = "DGD")]
    Dgd,
    #[sea_orm(string_value = "CI")]
    Ci,
    #[sea_orm(string_value = "MSDS")]
    Msds,
}

/// 화물과 v2 문서 버전의 역할별 다대다 관계
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cargo_item_documents")]
pub struct Model {
    /// 화물 문서 관계 고유번호
    #[sea_orm(primary_key)]
    pub cargo_item_document_id: i64,
    /// 레거시 화물 FK
    pub cargo_item_id: i64,
    /// v2 화물문서 버전 FK
    pub cargo_document_version_id: i64,
    /// 화물에서 문서가 담당하는 역할
    pub document_role: CargoDocumentRole,
    /// 동일 역할의 대표 문서 여부
    #[sea_orm(default_value = false)]
    pub is_primary: bool,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "cargo_item_id", to = "cargo_item_id")]
    pub cargo_item: HasOne<super::cargo_items::Entity>,
    #[sea_orm(belongs_to, from = "cargo_document_version_id", to = "cargo_document_version_id")]
    pub cargo_document_version: HasOne<super::cargo_document_versions::Entity>,
}


/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["cargo_item_id", "cargo_document_version_id", "document_role"], // uq_cargo_item_document_role
];
vespera::schema_type!(Schema from Model, name = "CargoItemDocumentsSchema");
impl ActiveModelBehavior for ActiveModel {}
