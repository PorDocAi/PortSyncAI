use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "work_assignments_eligibility_status"
)]
pub enum EligibilityStatus {
    #[sea_orm(string_value = "ELIGIBLE")]
    Eligible,
    #[sea_orm(string_value = "EXCLUDED")]
    Excluded,
}

/// 작업 배치 (FR-F3 — 투입 가능/제외 필터 결과)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "work_assignments")]
pub struct Model {
    /// 배치 고유번호
    #[sea_orm(primary_key)]
    pub assignment_id: i64,
    /// 작업자 FK
    #[sea_orm(indexed)]
    pub employee_id: i64,
    /// 작업 일자
    #[sea_orm(indexed)]
    pub work_date: Date,
    /// 대상 화물 FK
    pub cargo_item_id: Option<i64>,
    /// 배치 담당자 FK
    pub assigned_by_id: i64,
    /// 투입 가능 여부 (제한규칙 대조 결과)
    #[sea_orm(default_value = "ELIGIBLE")]
    pub eligibility_status: EligibilityStatus,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(
        belongs_to,
        relation_enum = "Employee",
        from = "employee_id",
        to = "employee_id"
    )]
    pub employee: HasOne<super::employees::Entity>,
    #[sea_orm(belongs_to, from = "cargo_item_id", to = "cargo_item_id")]
    pub cargo_item: HasOne<super::cargo_items::Entity>,
    #[sea_orm(
        belongs_to,
        relation_enum = "AssignedBy",
        from = "assigned_by_id",
        to = "employee_id"
    )]
    pub assigned_by: HasOne<super::employees::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [employee_id]
// (unnamed) on [work_date]
vespera::schema_type!(Schema from Model, name = "WorkAssignmentsSchema");
impl ActiveModelBehavior for ActiveModel {}
