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

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "cargo_documents_review_status"
)]
pub enum ReviewStatus {
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "CONFIRMED")]
    Confirmed,
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
    /// 사용자가 업로드한 원본 파일명
    pub original_file_name: Option<String>,
    /// 업로드 MIME 타입
    pub content_type: Option<String>,
    /// 파일 크기(byte)
    pub file_size: Option<i64>,
    /// SHA-256 파일 무결성 해시
    pub file_hash: Option<String>,
    /// 검수 상태 (FR-B2 — CONFIRMED여야 작업 배정 가능)
    #[sea_orm(default_value = "PENDING")]
    pub review_status: ReviewStatus,
    /// 업로더 FK
    pub uploaded_by_id: i64,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "uploaded_by_id", to = "employee_id")]
    pub uploaded_by: HasOne<super::employees::Entity>,
    #[sea_orm(has_many)]
    pub cargo_items: HasMany<super::cargo_items::Entity>,
    #[sea_orm(has_one)]
    pub cargo_document_versions: HasOne<super::cargo_document_versions::Entity>,
}

vespera::schema_type!(Schema from Model, name = "CargoDocumentsSchema");
impl ActiveModelBehavior for ActiveModel {}
