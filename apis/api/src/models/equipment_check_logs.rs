use sea_orm::entity::prelude::*;

/// 당일 장비 NFC 태깅 기록 (FR-D3/D4 — 돌려쓰기 방지)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "equipment_check_logs")]
pub struct Model {
    /// 태깅 기록 고유번호
    #[sea_orm(primary_key)]
    pub check_log_id: i64,
    /// 출근 FK
    #[sea_orm(indexed)]
    pub attendance_id: i64,
    /// 직원 FK
    pub employee_id: i64,
    /// 장비 FK
    pub equipment_id: i64,
    /// 작업 일자 (장비+일자 유니크로 중복사용 차단, FR-D4)
    pub work_date: Date,
    /// 태깅 시각
    #[sea_orm(default_value = "NOW()")]
    pub tagged_at: DateTimeWithTimeZone,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "attendance_id", to = "attendance_id")]
    pub attendance: HasOne<super::attendances::Entity>,
    #[sea_orm(belongs_to, from = "employee_id", to = "employee_id")]
    pub employee: HasOne<super::employees::Entity>,
    #[sea_orm(belongs_to, from = "equipment_id", to = "equipment_id")]
    pub equipment: HasOne<super::equipment::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [attendance_id]

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["equipment_id", "work_date"], // uq_equipment_workdate
];
vespera::schema_type!(Schema from Model, name = "EquipmentCheckLogsSchema");
impl ActiveModelBehavior for ActiveModel {}
