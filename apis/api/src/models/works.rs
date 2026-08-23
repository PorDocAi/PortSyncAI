use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "works_work_lifecycle_status"
)]
pub enum WorkLifecycleStatus {
    #[sea_orm(string_value = "PLANNED")]
    Planned,
    #[sea_orm(string_value = "ACTIVE")]
    Active,
    #[sea_orm(string_value = "STOPPED")]
    Stopped,
    #[sea_orm(string_value = "COMPLETED")]
    Completed,
    #[sea_orm(string_value = "CANCELLED")]
    Cancelled,
}

/// 컨테이너·화물 대상을 묶는 작업 단위
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "works")]
pub struct Model {
    /// 작업 고유번호
    #[sea_orm(primary_key)]
    pub work_id: i64,
    /// 작업 유형 FK
    #[sea_orm(indexed)]
    pub work_type_id: i64,
    /// 운영 작업 참조번호
    #[sea_orm(unique)]
    pub work_reference: String,
    /// 작업 수명주기 상태
    #[sea_orm(indexed, default_value = "PLANNED")]
    pub status: WorkLifecycleStatus,
    /// 예정 시작일시
    pub scheduled_start_at: DateTimeWithTimeZone,
    /// 예정 종료일시
    pub scheduled_end_at: Option<DateTimeWithTimeZone>,
    /// 실제 시작일시
    pub started_at: Option<DateTimeWithTimeZone>,
    /// 완료일시
    pub completed_at: Option<DateTimeWithTimeZone>,
    /// 작업 등록자 FK
    pub created_by_id: i64,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "work_type_id", to = "work_type_id")]
    pub work_type: HasOne<super::work_types::Entity>,
    #[sea_orm(belongs_to, from = "created_by_id", to = "employee_id")]
    pub created_by: HasOne<super::employees::Entity>,
    #[sea_orm(has_many)]
    pub work_ppe_requirement_snapshots: HasMany<super::work_ppe_requirement_snapshots::Entity>,
    #[sea_orm(has_many)]
    pub v2_work_assignments: HasMany<super::v2_work_assignments::Entity>,
    #[sea_orm(has_many)]
    pub work_stops: HasMany<super::work_stops::Entity>,
    #[sea_orm(has_many)]
    pub work_targets: HasMany<super::work_targets::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [work_type_id]
// (unnamed) on [status]
vespera::schema_type!(Schema from Model, name = "WorksSchema");
impl ActiveModelBehavior for ActiveModel {}
