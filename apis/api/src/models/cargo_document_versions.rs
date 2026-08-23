use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "cargo_document_versions_v2_cargo_document_type")]
pub enum V2CargoDocumentType {
    #[sea_orm(string_value = "BL")]
    Bl,
    #[sea_orm(string_value = "DGD")]
    Dgd,
    #[sea_orm(string_value = "CI")]
    Ci,
    #[sea_orm(string_value = "MSDS")]
    Msds,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "cargo_document_versions_cargo_processing_status")]
pub enum CargoProcessingStatus {
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "PROCESSING")]
    Processing,
    #[sea_orm(string_value = "COMPLETED")]
    Completed,
    #[sea_orm(string_value = "FAILED")]
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "cargo_document_versions_cargo_review_status")]
pub enum CargoReviewStatus {
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "CONFIRMED")]
    Confirmed,
    #[sea_orm(string_value = "REJECTED")]
    Rejected,
}

/// 화물 문서의 v2 유형·처리·검수 버전 상태
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cargo_document_versions")]
pub struct Model {
    /// v2 화물문서 버전 고유번호
    #[sea_orm(primary_key)]
    pub cargo_document_version_id: i64,
    /// 변환된 0001-0004 화물문서 FK, v2 신규 문서는 null
    #[sea_orm(unique)]
    pub legacy_cargo_document_id: Option<i64>,
    /// v2 문서 유형
    pub document_type: V2CargoDocumentType,
    /// 동일 문서의 검수 버전
    #[sea_orm(default_value = 1)]
    pub document_version: i32,
    /// 문서 분석 처리 상태
    #[sea_orm(indexed, default_value = "PENDING")]
    pub processing_status: CargoProcessingStatus,
    /// v2 관리자 검수 상태
    #[sea_orm(indexed, default_value = "PENDING")]
    pub review_status: CargoReviewStatus,
    /// 검수자 FK
    pub reviewed_by_id: Option<i64>,
    /// 검수일시
    pub reviewed_at: Option<DateTimeWithTimeZone>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "legacy_cargo_document_id", to = "cargo_document_id")]
    pub legacy: HasOne<super::cargo_documents::Entity>,
    #[sea_orm(belongs_to, from = "reviewed_by_id", to = "employee_id")]
    pub reviewed_by: HasOne<super::employees::Entity>,
    #[sea_orm(has_many)]
    pub ppe_requirements: HasMany<super::ppe_requirements::Entity>,
    #[sea_orm(has_many)]
    pub work_ppe_requirement_snapshots: HasMany<super::work_ppe_requirement_snapshots::Entity>,
    #[sea_orm(has_many)]
    pub cargo_item_documents: HasMany<super::cargo_item_documents::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [processing_status]
// (unnamed) on [review_status]
vespera::schema_type!(Schema from Model, name = "CargoDocumentVersionsSchema");
impl ActiveModelBehavior for ActiveModel {}
