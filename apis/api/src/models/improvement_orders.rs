use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "improvement_orders_improvement_status")]
pub enum ImprovementStatus {
    #[sea_orm(string_value = "ISSUED")]
    Issued,
    #[sea_orm(string_value = "ACTION_TAKEN")]
    ActionTaken,
    #[sea_orm(string_value = "COMPLETED")]
    Completed,
}

/// 항만안전점검관 개선명령 워크플로 (FR-E3, 3단계)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "improvement_orders")]
pub struct Model {
    /// 개선명령 고유번호
    #[sea_orm(primary_key)]
    pub order_id: i64,
    /// 명령 등록자 FK (관리자/점검관)
    pub issued_by_id: i64,
    /// 대상 작업자 FK
    pub target_employee_id: Option<i64>,
    /// 관련 출근 FK
    pub attendance_id: Option<i64>,
    /// 개선명령 내용
    pub content: String,
    /// 진행 상태 (등록→조치→완료)
    #[sea_orm(default_value = "ISSUED")]
    pub status: ImprovementStatus,
    /// 조치 사진 경로 (FR-E3)
    pub action_photo_url: Option<String>,
    /// 조치 시각
    pub action_taken_at: Option<DateTimeWithTimeZone>,
    /// 완료 보고 시각
    pub completed_at: Option<DateTimeWithTimeZone>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, relation_enum = "IssuedBy", from = "issued_by_id", to = "employee_id")]
    pub issued_by: HasOne<super::employees::Entity>,
    #[sea_orm(belongs_to, relation_enum = "TargetEmployee", from = "target_employee_id", to = "employee_id")]
    pub target: HasOne<super::employees::Entity>,
    #[sea_orm(belongs_to, from = "attendance_id", to = "attendance_id")]
    pub attendance: HasOne<super::attendances::Entity>,
}

vespera::schema_type!(Schema from Model, name = "ImprovementOrdersSchema");
impl ActiveModelBehavior for ActiveModel {}
