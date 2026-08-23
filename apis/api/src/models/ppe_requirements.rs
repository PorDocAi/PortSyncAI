use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "ppe_requirements_ppe_review_status"
)]
pub enum PpeReviewStatus {
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "CONFIRMED")]
    Confirmed,
    #[sea_orm(string_value = "REJECTED")]
    Rejected,
}

/// 검수된 문서 원문 기반 보호구 요구조건
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ppe_requirements")]
pub struct Model {
    /// 보호구 요구조건 고유번호
    #[sea_orm(primary_key)]
    pub ppe_requirement_id: i64,
    /// 요구 장비 종류 FK
    #[sea_orm(indexed)]
    pub equipment_type_id: i64,
    /// 보호구 카테고리
    #[sea_orm(indexed)]
    pub category: String,
    /// 성능조건 구조화 값
    pub performance_criteria: Option<Json>,
    /// 근거 문서 원문
    pub source_text: String,
    /// 근거 v2 화물문서 버전 FK
    #[sea_orm(indexed)]
    pub source_document_version_id: i64,
    /// 근거 문서 버전
    pub source_document_version_number: i32,
    /// 요구조건 검수 상태
    #[sea_orm(default_value = "PENDING")]
    pub review_status: PpeReviewStatus,
    /// 검수자 FK
    pub reviewed_by_id: Option<i64>,
    /// 검수일시
    pub reviewed_at: Option<DateTimeWithTimeZone>,
    /// 사용 여부
    #[sea_orm(default_value = true)]
    pub is_active: bool,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "equipment_type_id", to = "equipment_type_id")]
    pub equipment_type: HasOne<super::equipment_types::Entity>,
    #[sea_orm(
        belongs_to,
        from = "source_document_version_id",
        to = "cargo_document_version_id"
    )]
    pub source_document_version: HasOne<super::cargo_document_versions::Entity>,
    #[sea_orm(belongs_to, from = "reviewed_by_id", to = "employee_id")]
    pub reviewed_by: HasOne<super::employees::Entity>,
    #[sea_orm(has_many)]
    pub work_ppe_requirement_snapshots: HasMany<super::work_ppe_requirement_snapshots::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [equipment_type_id]
// (unnamed) on [category]
// (unnamed) on [source_document_version_id]
vespera::schema_type!(Schema from Model, name = "PpeRequirementsSchema");
impl ActiveModelBehavior for ActiveModel {}
