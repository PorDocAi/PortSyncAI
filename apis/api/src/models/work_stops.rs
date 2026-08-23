use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "work_stops_work_stop_status")]
pub enum WorkStopStatus {
    #[sea_orm(string_value = "OPEN")]
    Open,
    #[sea_orm(string_value = "CLOSED")]
    Closed,
}

/// 작업 또는 작업 배정 범위의 작업중지 기록
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "work_stops")]
pub struct Model {
    /// 작업중지 고유번호
    #[sea_orm(primary_key)]
    pub work_stop_id: i64,
    /// 중지 대상 작업 FK, 레거시 변환행은 null
    #[sea_orm(indexed)]
    pub work_id: Option<i64>,
    /// 중지 대상 v2 작업 배정 FK
    #[sea_orm(indexed)]
    pub work_assignment_id: Option<i64>,
    /// PR #48 출근 작업중지 변환 출처 FK
    #[sea_orm(unique)]
    pub legacy_attendance_id: Option<i64>,
    /// 작업중지 상태
    #[sea_orm(indexed, default_value = "OPEN")]
    pub status: WorkStopStatus,
    /// 작업중지 사유
    pub reason: String,
    /// 작업중지 등록자 FK, PR #48 변환행은 null
    pub stopped_by_id: Option<i64>,
    /// 작업중지 일시
    #[sea_orm(default_value = "NOW()")]
    pub stopped_at: DateTimeWithTimeZone,
    /// 작업중지 해제자 FK
    pub closed_by_id: Option<i64>,
    /// 작업중지 해제일시
    pub closed_at: Option<DateTimeWithTimeZone>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "work_id", to = "work_id")]
    pub work: HasOne<super::works::Entity>,
    #[sea_orm(belongs_to, from = "work_assignment_id", to = "work_assignment_id")]
    pub work_assignment: HasOne<super::v2_work_assignments::Entity>,
    #[sea_orm(belongs_to, from = "legacy_attendance_id", to = "attendance_id")]
    pub legacy: HasOne<super::attendances::Entity>,
    #[sea_orm(belongs_to, relation_enum = "StoppedBy", from = "stopped_by_id", to = "employee_id")]
    pub stopped_by: HasOne<super::employees::Entity>,
    #[sea_orm(belongs_to, relation_enum = "ClosedBy", from = "closed_by_id", to = "employee_id")]
    pub closed_by: HasOne<super::employees::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [work_id]
// (unnamed) on [work_assignment_id]
// (unnamed) on [status]
vespera::schema_type!(Schema from Model, name = "WorkStopsSchema");
impl ActiveModelBehavior for ActiveModel {}
