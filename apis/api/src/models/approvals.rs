use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "approvals_approval_decision")]
pub enum ApprovalDecision {
    #[sea_orm(string_value = "APPROVED")]
    Approved,
    #[sea_orm(string_value = "REJECTED")]
    Rejected,
}

/// 관리자 승인/반려 기록 (FR-E1/E2)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "approvals")]
pub struct Model {
    /// 승인 고유번호
    #[sea_orm(primary_key)]
    pub approval_id: i64,
    /// 출근 FK
    #[sea_orm(indexed)]
    pub attendance_id: i64,
    /// 승인자 FK
    pub approver_id: i64,
    /// 승인/반려
    pub decision: ApprovalDecision,
    /// 사유 (FR-E2)
    pub reason: Option<String>,
    /// 결정 시각
    #[sea_orm(default_value = "NOW()")]
    pub decided_at: DateTimeWithTimeZone,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "attendance_id", to = "attendance_id")]
    pub attendance: HasOne<super::attendances::Entity>,
    #[sea_orm(belongs_to, from = "approver_id", to = "employee_id")]
    pub approver: HasOne<super::employees::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [attendance_id]
vespera::schema_type!(Schema from Model, name = "ApprovalsSchema");
impl ActiveModelBehavior for ActiveModel {}
