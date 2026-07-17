use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "employees_system_role"
)]
pub enum SystemRole {
    #[sea_orm(string_value = "ADMIN")]
    Admin,
    #[sea_orm(string_value = "SAFETY_MANAGER")]
    SafetyManager,
    #[sea_orm(string_value = "SUPERVISOR")]
    Supervisor,
    #[sea_orm(string_value = "WORKER")]
    Worker,
}

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "employees_employee_status"
)]
pub enum EmployeeStatus {
    #[sea_orm(string_value = "ACTIVE")]
    Active,
    #[sea_orm(string_value = "ON_LEAVE")]
    OnLeave,
    #[sea_orm(string_value = "SUSPENDED")]
    Suspended,
    #[sea_orm(string_value = "RESIGNED")]
    Resigned,
}

/// 직원
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "employees")]
pub struct Model {
    /// 직원 고유번호 (내부 식별자)
    #[sea_orm(primary_key)]
    pub employee_id: i64,
    /// 회사 표준 사번
    #[sea_orm(unique)]
    pub employee_number: String,
    /// 이름
    pub name: String,
    /// 사내 이메일
    #[sea_orm(unique)]
    pub email: String,
    /// bcrypt/argon2 해시
    pub password_hash: String,
    /// 전화번호
    pub phone_number: Option<String>,
    /// 부서 FK
    #[sea_orm(indexed)]
    pub department_id: i64,
    /// 직무 FK
    #[sea_orm(indexed)]
    pub job_role_id: i64,
    /// 직급 (사원/대리/과장 등)
    pub position: Option<String>,
    /// 시스템 권한
    #[sea_orm(default_value = "WORKER")]
    pub system_role: SystemRole,
    /// 모국어 코드 (번역/TTS 기준, FR-C1)
    #[sea_orm(default_value = "ko")]
    pub preferred_language: String,
    /// 사원증 NFC UID (게이트 태깅 식별자)
    #[sea_orm(unique)]
    pub nfc_card_uid: Option<String>,
    /// 사원증 발급일
    pub nfc_card_issued_at: Option<DateTimeWithTimeZone>,
    /// 입사일
    pub hire_date: Date,
    /// 상태
    #[sea_orm(default_value = "ACTIVE")]
    pub status: EmployeeStatus,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    /// 퇴사일시
    pub resigned_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "department_id", to = "department_id")]
    pub department: HasOne<super::departments::Entity>,
    #[sea_orm(belongs_to, from = "job_role_id", to = "job_role_id")]
    pub job_role: HasOne<super::job_roles::Entity>,
    #[sea_orm(has_many)]
    pub approvals: HasMany<super::approvals::Entity>,
    #[sea_orm(has_many)]
    pub attendances: HasMany<super::attendances::Entity>,
    #[sea_orm(has_many)]
    pub audit_logs: HasMany<super::audit_logs::Entity>,
    #[sea_orm(has_many)]
    pub cargo_documents: HasMany<super::cargo_documents::Entity>,
    #[sea_orm(has_one)]
    pub employee_health_profiles: HasOne<super::employee_health_profiles::Entity>,
    #[sea_orm(has_many)]
    pub equipment_check_logs: HasMany<super::equipment_check_logs::Entity>,
    #[sea_orm(has_many, relation_enum = "ImprovementOrders", via_rel = "IssuedBy")]
    pub issued_by_improvement_orders: HasMany<super::improvement_orders::Entity>,
    #[sea_orm(has_many, relation_enum = "TargetEmployee", via_rel = "TargetEmployee")]
    pub target_employee_improvement_orders: HasMany<super::improvement_orders::Entity>,
    #[sea_orm(has_many)]
    pub instruction_acknowledgements: HasMany<super::instruction_acknowledgements::Entity>,
    #[sea_orm(has_many)]
    pub regulation_documents: HasMany<super::regulation_documents::Entity>,
    #[sea_orm(has_many, relation_enum = "WorkAssignments", via_rel = "Employee")]
    pub employee_work_assignments: HasMany<super::work_assignments::Entity>,
    #[sea_orm(has_many, relation_enum = "AssignedBy", via_rel = "AssignedBy")]
    pub assigned_by_work_assignments: HasMany<super::work_assignments::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [department_id]
// (unnamed) on [job_role_id]
vespera::schema_type!(Schema from Model, name = "EmployeesSchema");
impl ActiveModelBehavior for ActiveModel {}
