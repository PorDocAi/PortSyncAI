use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "work_preparations_work_preparation_status"
)]
pub enum WorkPreparationStatus {
    #[sea_orm(string_value = "NOT_STARTED")]
    NotStarted,
    #[sea_orm(string_value = "IN_PROGRESS")]
    InProgress,
    #[sea_orm(string_value = "READY")]
    Ready,
    #[sea_orm(string_value = "BLOCKED")]
    Blocked,
}

/// 출근 전역 불리언을 대체하는 작업 배정별 준비 상태
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "work_preparations")]
pub struct Model {
    /// 작업 준비 고유번호
    #[sea_orm(primary_key)]
    pub work_preparation_id: i64,
    /// v2 작업 배정 FK
    #[sea_orm(unique)]
    pub work_assignment_id: i64,
    /// 종합 준비 상태
    #[sea_orm(indexed, default_value = "NOT_STARTED")]
    pub status: WorkPreparationStatus,
    /// 관련 문서 검수 충족 여부
    #[sea_orm(default_value = false)]
    pub document_ready: bool,
    /// 교육 이수 충족 여부
    #[sea_orm(default_value = false)]
    pub education_ready: bool,
    /// 작업 지침 확인 여부
    #[sea_orm(default_value = false)]
    pub instruction_ready: bool,
    /// 보호구 준비 충족 여부
    #[sea_orm(default_value = false)]
    pub ppe_ready: bool,
    /// 준비 판정 입력 버전
    pub decision_input_versions: Json,
    /// 준비 완료일시
    pub prepared_at: Option<DateTimeWithTimeZone>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "work_assignment_id", to = "work_assignment_id")]
    pub work_assignment: HasOne<super::v2_work_assignments::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [status]
vespera::schema_type!(Schema from Model, name = "WorkPreparationsSchema");
impl ActiveModelBehavior for ActiveModel {}
