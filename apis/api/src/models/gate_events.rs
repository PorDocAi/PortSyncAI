use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "gate_events_gate_decision"
)]
pub enum GateDecision {
    #[sea_orm(string_value = "PASS")]
    Pass,
    #[sea_orm(string_value = "BLOCK")]
    Block,
}

/// 작업 배정 준비도를 기록하는 append-only 게이트 PASS/BLOCK 이벤트
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "gate_events")]
pub struct Model {
    /// 게이트 이벤트 고유번호
    #[sea_orm(primary_key)]
    pub gate_event_id: i64,
    /// 검증 단말 FK
    #[sea_orm(indexed)]
    pub terminal_id: Option<i64>,
    /// 판정 시점 게이트 식별자
    pub gate_id: String,
    /// 판정 v2 작업 배정 FK, 변환 불가 레거시는 null
    #[sea_orm(indexed)]
    pub work_assignment_id: Option<i64>,
    /// 판정 작업자 FK
    #[sea_orm(indexed)]
    pub employee_id: i64,
    /// 단말 범위 멱등키, 레거시 변환행은 null
    pub idempotency_key: Option<Uuid>,
    /// 요청 SHA-256 지문, 레거시 변환행은 null
    pub request_fingerprint: Option<String>,
    /// 통과 또는 차단 판정
    #[sea_orm(indexed)]
    pub decision: GateDecision,
    /// 우선순위가 가장 높은 안정된 판정 코드
    pub reason_code: String,
    /// 우선순위대로 정렬된 전체 판정 코드
    pub reason_codes: Json,
    /// 판정에 사용한 문서·교육·보호구·중지 버전
    pub decision_input_versions: Json,
    /// 최초 응답 HTTP 상태
    pub http_status: i16,
    /// 멱등 재전송할 최초 정확 응답 JSON
    pub response_json: Json,
    /// 서버 판정 발생일시
    #[sea_orm(default_value = "NOW()")]
    pub occurred_at: DateTimeWithTimeZone,
    /// PR #48 게이트 검증 기록 변환 출처 FK
    #[sea_orm(unique)]
    pub legacy_verify_log_id: Option<i64>,
    /// PR #48 출근 참조 FK
    pub legacy_attendance_id: Option<i64>,
    /// PR #48 작업일 감사값
    pub legacy_work_date: Option<Date>,
    #[sea_orm(belongs_to, from = "terminal_id", to = "terminal_id")]
    pub terminal: HasOne<super::gate_terminals::Entity>,
    #[sea_orm(belongs_to, from = "work_assignment_id", to = "work_assignment_id")]
    pub work_assignment: HasOne<super::v2_work_assignments::Entity>,
    #[sea_orm(belongs_to, from = "employee_id", to = "employee_id")]
    pub employee: HasOne<super::employees::Entity>,
    #[sea_orm(belongs_to, from = "legacy_verify_log_id", to = "verify_log_id")]
    pub legacy: HasOne<super::gate_verify_logs::Entity>,
    #[sea_orm(belongs_to, from = "legacy_attendance_id", to = "attendance_id")]
    pub legacy_1: HasOne<super::attendances::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [terminal_id]
// (unnamed) on [work_assignment_id]
// (unnamed) on [employee_id]
// (unnamed) on [decision]

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["terminal_id", "idempotency_key"], // uq_gate_event_terminal_idempotency
];
vespera::schema_type!(Schema from Model, name = "GateEventsSchema");
impl ActiveModelBehavior for ActiveModel {}
