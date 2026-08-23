use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "attendances_approval_status")]
pub enum ApprovalStatus {
    #[sea_orm(string_value = "NOT_REQUIRED")]
    NotRequired,
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "APPROVED")]
    Approved,
    #[sea_orm(string_value = "REJECTED")]
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "attendances_gate_status")]
pub enum GateStatus {
    #[sea_orm(string_value = "BLOCKED")]
    Blocked,
    #[sea_orm(string_value = "READY")]
    Ready,
    #[sea_orm(string_value = "PASSED")]
    Passed,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "attendances_work_status")]
pub enum WorkStatus {
    #[sea_orm(string_value = "NORMAL")]
    Normal,
    #[sea_orm(string_value = "STOPPED")]
    Stopped,
}

/// 출근/게이트 통과 상태 (FR-C4, D5 — 인지·장비·승인 집계)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "attendances")]
pub struct Model {
    /// 출근 고유번호
    #[sea_orm(primary_key)]
    pub attendance_id: i64,
    /// 직원 FK
    pub employee_id: i64,
    /// 작업 일자
    #[sea_orm(indexed)]
    pub work_date: Date,
    /// 안전지침 확인 완료 (FR-C4)
    #[sea_orm(default_value = false)]
    pub instruction_ack_completed: bool,
    /// 필수 장비 착용 완료 (FR-D3)
    #[sea_orm(default_value = false)]
    pub equipment_check_completed: bool,
    /// 관리자 승인 상태
    #[sea_orm(default_value = "NOT_REQUIRED")]
    pub approval_status: ApprovalStatus,
    /// 게이트 통과 상태
    #[sea_orm(default_value = "BLOCKED")]
    pub gate_status: GateStatus,
    /// 작업중지 여부 (시나리오 5 — STOPPED면 게이트 차단)
    #[sea_orm(default_value = "NORMAL")]
    pub work_status: WorkStatus,
    /// 사원증 태깅 통과 시각 (FR-D5)
    pub gate_passed_at: Option<DateTimeWithTimeZone>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "employee_id", to = "employee_id")]
    pub employee: HasOne<super::employees::Entity>,
    #[sea_orm(has_many)]
    pub equipment_check_logs: HasMany<super::equipment_check_logs::Entity>,
    #[sea_orm(has_many)]
    pub gate_verify_logs: HasMany<super::gate_verify_logs::Entity>,
    #[sea_orm(has_many)]
    pub gate_events: HasMany<super::gate_events::Entity>,
    #[sea_orm(has_one)]
    pub work_stops: HasOne<super::work_stops::Entity>,
    #[sea_orm(has_many)]
    pub approvals: HasMany<super::approvals::Entity>,
    #[sea_orm(has_many)]
    pub instruction_acknowledgements: HasMany<super::instruction_acknowledgements::Entity>,
    #[sea_orm(has_many)]
    pub improvement_orders: HasMany<super::improvement_orders::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [work_date]

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["employee_id", "work_date"], // uq_employee_workdate
];
vespera::schema_type!(Schema from Model, name = "AttendancesSchema");
impl ActiveModelBehavior for ActiveModel {}
