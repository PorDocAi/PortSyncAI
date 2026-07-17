use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "cargo_documents_cargo_document_type"
)]
pub enum CargoDocumentType {
    #[sea_orm(string_value = "BL")]
    Bl,
    #[sea_orm(string_value = "DGD")]
    Dgd,
}

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "cargo_documents_file_format"
)]
pub enum FileFormat {
    #[sea_orm(string_value = "HWP")]
    Hwp,
    #[sea_orm(string_value = "PDF")]
    Pdf,
    #[sea_orm(string_value = "XLSX")]
    Xlsx,
    #[sea_orm(string_value = "DOCX")]
    Docx,
    #[sea_orm(string_value = "IMAGE")]
    Image,
}

/// 화물 문서 원문 (B/L·DGD 업로드, FR-B)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cargo_documents")]
pub struct Model {
    /// 화물문서 고유번호
    #[sea_orm(primary_key)]
    pub cargo_document_id: i64,
    /// 문서 유형 (선하증권/위험물신고서)
    pub document_type: CargoDocumentType,
    /// 파일 형식
    pub file_format: FileFormat,
    /// 원본 파일 경로
    pub file_url: String,
    /// 업로더 FK
    pub uploaded_by_id: i64,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "uploaded_by_id", to = "employee_id")]
    pub uploaded_by: HasOne<super::employees::Entity>,
    #[sea_orm(has_many)]
    pub cargo_items: HasMany<super::cargo_items::Entity>,
}

vespera::schema_type!(Schema from Model, name = "CargoDocumentsSchema");
impl ActiveModelBehavior for ActiveModel {}
