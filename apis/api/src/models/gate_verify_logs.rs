use sea_orm::entity::prelude::*;

/// 게이트 검증 기록 (FR-D5, AC-5 — 이벤트 멱등 보장)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "gate_verify_logs")]
pub struct Model {
    /// 게이트 검증 기록 고유번호
    #[sea_orm(primary_key)]
    pub verify_log_id: i64,
    /// 출근 FK (출근 절차 시작 전 스캔은 NULL)
    #[sea_orm(indexed)]
    pub attendance_id: Option<i64>,
    /// 직원 FK
    pub employee_id: i64,
    /// 검증 단말 FK (게이트 단말 자격증명)
    pub terminal_id: Option<i64>,
    /// 통과 허용 여부
    #[sea_orm(default_value = false)]
    pub allowed: bool,
    /// 판정 사유 (한국어 안내 문구)
    pub reason: String,
    /// 작업 일자 (출근일 기준 통과 이벤트 멱등키 구성 요소)
    pub work_date: Date,
    /// 통과 이벤트 멱등 플래그 — PASSED 건만 true로 기록해 유니크 제약 대상이 된다
    #[sea_orm(default_value = false)]
    pub is_pass_event: bool,
    /// 기록 시각
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "attendance_id", to = "attendance_id")]
    pub attendance: HasOne<super::attendances::Entity>,
    #[sea_orm(belongs_to, from = "employee_id", to = "employee_id")]
    pub employee: HasOne<super::employees::Entity>,
    #[sea_orm(belongs_to, from = "terminal_id", to = "terminal_id")]
    pub terminal: HasOne<super::gate_terminals::Entity>,
    #[sea_orm(has_one)]
    pub gate_events: HasOne<super::gate_events::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [attendance_id]

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["work_date", "is_pass_event"], // uq_attendance_workdate_passed
];
vespera::schema_type!(Schema from Model, name = "GateVerifyLogsSchema");
impl ActiveModelBehavior for ActiveModel {}
