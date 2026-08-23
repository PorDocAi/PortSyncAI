use sea_orm::entity::prelude::*;

/// 작업 확정 시점의 불변 보호구 요구조건 스냅샷
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "work_ppe_requirement_snapshots")]
pub struct Model {
    /// 작업 보호구 스냅샷 고유번호
    #[sea_orm(primary_key)]
    pub work_ppe_requirement_snapshot_id: i64,
    /// 작업 FK
    pub work_id: i64,
    /// 원본 보호구 요구조건 FK
    pub ppe_requirement_id: i64,
    /// 확정 장비 종류 FK
    #[sea_orm(indexed)]
    pub equipment_type_id: i64,
    /// 확정 보호구 카테고리
    pub category: String,
    /// 확정 성능조건
    pub performance_criteria: Option<Json>,
    /// 확정 근거 원문
    pub source_text: String,
    /// 확정 근거 v2 문서 버전 FK
    pub source_document_version_id: i64,
    /// 확정 근거 문서 버전
    pub source_document_version_number: i32,
    /// 확정 검수자 FK
    pub reviewed_by_id: i64,
    /// 원본 검수일시
    pub reviewed_at: DateTimeWithTimeZone,
    /// 스냅샷 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub snapshotted_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "work_id", to = "work_id")]
    pub work: HasOne<super::works::Entity>,
    #[sea_orm(belongs_to, from = "ppe_requirement_id", to = "ppe_requirement_id")]
    pub ppe_requirement: HasOne<super::ppe_requirements::Entity>,
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
    pub work_assignment_equipments: HasMany<super::work_assignment_equipment::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [equipment_type_id]

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["work_id", "ppe_requirement_id"], // uq_work_ppe_requirement_snapshot
];
vespera::schema_type!(Schema from Model, name = "WorkPpeRequirementSnapshotsSchema");
impl ActiveModelBehavior for ActiveModel {}
