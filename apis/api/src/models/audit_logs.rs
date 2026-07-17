use sea_orm::entity::prelude::*;

/// 감사 로그 (FR-G1, append-only — 누가/언제/무엇을/어떤 버전, NFR 무결성)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_logs")]
pub struct Model {
    /// 감사 로그 고유번호
    #[sea_orm(primary_key)]
    pub audit_log_id: i64,
    /// 행위자 FK (시스템 이벤트는 null)
    #[sea_orm(indexed)]
    pub actor_employee_id: Option<i64>,
    /// 이벤트 유형 (INSTRUCTION_ACK/EQUIPMENT_CHECK/APPROVAL/GATE_PASS 등)
    #[sea_orm(indexed)]
    pub event_type: String,
    /// 대상 엔티티명
    pub target_type: Option<String>,
    /// 대상 레코드 PK
    pub target_id: Option<i64>,
    /// 관련 지침 버전 (FR-G1)
    pub instruction_version: Option<i32>,
    /// 상세 이벤트 데이터
    pub detail: Option<Json>,
    /// 발생 시각
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "actor_employee_id", to = "employee_id")]
    pub actor: HasOne<super::employees::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [actor_employee_id]
// (unnamed) on [event_type]
vespera::schema_type!(Schema from Model, name = "AuditLogsSchema");
impl ActiveModelBehavior for ActiveModel {}
