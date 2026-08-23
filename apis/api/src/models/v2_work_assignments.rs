use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "v2_work_assignments_v2_work_assignment_status")]
pub enum V2WorkAssignmentStatus {
    #[sea_orm(string_value = "ASSIGNED")]
    Assigned,
    #[sea_orm(string_value = "SELECTED")]
    Selected,
    #[sea_orm(string_value = "ACTIVE")]
    Active,
    #[sea_orm(string_value = "COMPLETED")]
    Completed,
    #[sea_orm(string_value = "CANCELLED")]
    Cancelled,
}

/// v2 작업과 작업자를 연결하는 authoritative 작업 배정
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "v2_work_assignments")]
pub struct Model {
    /// v2 작업 배정 고유번호
    #[sea_orm(primary_key)]
    pub work_assignment_id: i64,
    /// 변환된 0001 작업 배정 FK, v2 신규 배정은 null
    #[sea_orm(unique)]
    pub legacy_assignment_id: Option<i64>,
    /// v2 작업 FK
    pub work_id: i64,
    /// 작업자 FK
    #[sea_orm(indexed)]
    pub employee_id: i64,
    /// 배치 담당자 FK
    pub assigned_by_id: i64,
    /// v2 배정 수명주기 상태
    #[sea_orm(indexed, default_value = "ASSIGNED")]
    pub status: V2WorkAssignmentStatus,
    /// 작업자가 NFC 대상 작업으로 선택한 일시
    pub selected_at: Option<DateTimeWithTimeZone>,
    /// 배정 작업 시작일시
    pub started_at: Option<DateTimeWithTimeZone>,
    /// 배정 작업 완료일시
    pub completed_at: Option<DateTimeWithTimeZone>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "legacy_assignment_id", to = "assignment_id")]
    pub legacy: HasOne<super::work_assignments::Entity>,
    #[sea_orm(belongs_to, from = "work_id", to = "work_id")]
    pub work: HasOne<super::works::Entity>,
    #[sea_orm(belongs_to, relation_enum = "Employee", from = "employee_id", to = "employee_id")]
    pub employee: HasOne<super::employees::Entity>,
    #[sea_orm(belongs_to, relation_enum = "AssignedBy", from = "assigned_by_id", to = "employee_id")]
    pub assigned_by: HasOne<super::employees::Entity>,
    #[sea_orm(has_many)]
    pub work_assignment_equipments: HasMany<super::work_assignment_equipment::Entity>,
    #[sea_orm(has_many)]
    pub gate_events: HasMany<super::gate_events::Entity>,
    #[sea_orm(has_many)]
    pub equipment_check_events: HasMany<super::equipment_check_events::Entity>,
    #[sea_orm(has_many)]
    pub work_stops: HasMany<super::work_stops::Entity>,
    #[sea_orm(has_one)]
    pub work_preparations: HasOne<super::work_preparations::Entity>,
    #[sea_orm(has_many)]
    pub shared_equipment_claims: HasMany<super::shared_equipment_claims::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [employee_id]
// (unnamed) on [status]

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["work_id", "employee_id"], // uq_v2_work_assignment_work_employee
];
vespera::schema_type!(Schema from Model, name = "V2WorkAssignmentsSchema");
impl ActiveModelBehavior for ActiveModel {}
