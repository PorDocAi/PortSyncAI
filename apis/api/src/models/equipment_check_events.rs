use sea_orm::entity::prelude::*;

/// v2 작업 배정 범위의 append-only 장비 태깅·멱등 응답 이벤트
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "equipment_check_events")]
pub struct Model {
    /// 장비 태깅 이벤트 고유번호
    #[sea_orm(primary_key)]
    pub equipment_check_event_id: i64,
    /// JWT로 확인한 작업자 FK
    #[sea_orm(indexed)]
    pub employee_id: i64,
    /// 요청 v2 작업 배정 FK
    #[sea_orm(indexed)]
    pub work_assignment_id: i64,
    /// 토큰으로 확인한 v2 장비 프로필 FK, 미확인 토큰은 null
    pub equipment_profile_id: Option<i64>,
    /// 작업자 범위 멱등키
    pub idempotency_key: Uuid,
    /// 정규화하지 않은 요청의 SHA-256 지문
    pub request_fingerprint: String,
    /// 최초 응답 HTTP 상태
    pub http_status: i16,
    /// 장비 태깅 수락 여부
    pub accepted: bool,
    /// 안정된 결과 코드
    pub reason_code: String,
    /// 재전송할 최초 정확 응답 JSON
    pub response_json: Json,
    /// 클라이언트 스캔 감사시각
    pub client_scanned_at: DateTimeWithTimeZone,
    /// 서버 이벤트 발생일시
    #[sea_orm(default_value = "NOW()")]
    pub occurred_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "employee_id", to = "employee_id")]
    pub employee: HasOne<super::employees::Entity>,
    #[sea_orm(belongs_to, from = "work_assignment_id", to = "work_assignment_id")]
    pub work_assignment: HasOne<super::v2_work_assignments::Entity>,
    #[sea_orm(belongs_to, from = "equipment_profile_id", to = "equipment_profile_id")]
    pub equipment_profile: HasOne<super::equipment_profiles::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [employee_id]
// (unnamed) on [work_assignment_id]

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["employee_id", "idempotency_key"], // uq_equipment_check_employee_idempotency
];
vespera::schema_type!(Schema from Model, name = "EquipmentCheckEventsSchema");
impl ActiveModelBehavior for ActiveModel {}
