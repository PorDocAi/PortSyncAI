//! 게이트 검증 기록 (FR-D5, AC-5 — 이벤트 멱등 보장)
//!
//! 멱등 설계: `is_pass_event = true`인 행은 (attendance_id, work_date) 유니크 제약으로
//! 출근 건당 통과 이벤트를 최대 1건만 허용한다. 이미 PASSED 상태에서 verify_gate를
//! 재호출하면 INSERT가 제약에 걸려 상태 변경 없이 "이미 통과" 응답으로 멱등 동작하며,
//! 차단 응답은 is_pass_event = false로 남아 재태깅 시 사유를 다시 기록한다.
//! DB 유니크 제약이 최종 방어선이므로 동시 재요청에도 중복 통과 이벤트가 생기지 않는다.
use sea_orm::entity::prelude::*;

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
    #[sea_orm(unique_key = "uq_attendance_workdate_passed")]
    pub work_date: Date,
    /// 통과 이벤트 멱등 플래그 — PASSED 건만 true로 기록해 유니크 제약 대상이 된다
    #[sea_orm(default_value = false)]
    #[sea_orm(unique_key = "uq_attendance_workdate_passed")]
    pub is_pass_event: bool,
    /// 기록 시각
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "attendance_id", to = "attendance_id")]
    pub attendance: HasOne<super::attendances::Entity>,
    #[sea_orm(belongs_to, from = "employee_id", to = "employee_id")]
    pub employee: HasOne<super::employees::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [attendance_id]

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["attendance_id", "work_date", "is_pass_event"], // uq_attendance_workdate_passed
];
vespera::schema_type!(Schema from Model, name = "GateVerifyLogsSchema");
impl ActiveModelBehavior for ActiveModel {}
