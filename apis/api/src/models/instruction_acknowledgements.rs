use sea_orm::entity::prelude::*;

/// 안전수칙 인지 로그 (FR-C2/C3, append-only — 수정 불가, NFR 무결성)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "instruction_acknowledgements")]
pub struct Model {
    /// 인지 로그 고유번호
    #[sea_orm(primary_key)]
    pub acknowledgement_id: i64,
    /// 출근 FK
    #[sea_orm(indexed)]
    pub attendance_id: i64,
    /// 직원 FK
    #[sea_orm(indexed)]
    pub employee_id: i64,
    /// 안전지침 FK
    pub instruction_id: i64,
    /// 확인 당시 지침 버전 (FR-G1)
    pub instruction_version: i32,
    /// 표시된 언어
    pub language_code: String,
    /// 최하단 스크롤 완료 여부 (FR-C2)
    #[sea_orm(default_value = false)]
    pub scrolled_to_end: bool,
    /// 확인 시각 (FR-C3)
    #[sea_orm(default_value = "NOW()")]
    pub acknowledged_at: DateTimeWithTimeZone,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "attendance_id", to = "attendance_id")]
    pub attendance: HasOne<super::attendances::Entity>,
    #[sea_orm(belongs_to, from = "employee_id", to = "employee_id")]
    pub employee: HasOne<super::employees::Entity>,
    #[sea_orm(belongs_to, from = "instruction_id", to = "instruction_id")]
    pub instruction: HasOne<super::safety_instructions::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [attendance_id]
// (unnamed) on [employee_id]
vespera::schema_type!(Schema from Model, name = "InstructionAcknowledgementsSchema");
impl ActiveModelBehavior for ActiveModel {}
