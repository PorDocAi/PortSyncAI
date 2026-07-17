use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "safety_instructions_source_law"
)]
pub enum SourceLaw {
    #[sea_orm(string_value = "OSH_ACT")]
    OshAct,
    #[sea_orm(string_value = "PORT_SAFETY_ACT")]
    PortSafetyAct,
    #[sea_orm(string_value = "KOSHA_GUIDE")]
    KoshaGuide,
    #[sea_orm(string_value = "IMDG")]
    Imdg,
    #[sea_orm(string_value = "OTHER")]
    Other,
}

/// 안전지침 (문서에서 추출·요약, FR-A2/A4)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "safety_instructions")]
pub struct Model {
    /// 안전지침 고유번호
    #[sea_orm(primary_key)]
    pub instruction_id: i64,
    /// 출처 문서 FK (화물기반 지침은 null 가능)
    pub document_id: Option<i64>,
    /// 지침 제목
    pub title: String,
    /// 요약 구조화 텍스트
    pub summary: String,
    /// 위반 시 과태료 조항 (FR-A2)
    pub penalty_clause: Option<String>,
    /// 근거 법령
    pub source_law: SourceLaw,
    /// 지침 버전 (인지 로그에 기록, FR-G1)
    #[sea_orm(default_value = 1)]
    pub version: i32,
    /// 시행일
    pub effective_date: Option<Date>,
    /// 현행 여부
    #[sea_orm(default_value = true)]
    pub is_active: bool,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "document_id", to = "document_id")]
    pub document: HasOne<super::regulation_documents::Entity>,
    #[sea_orm(has_many)]
    pub class_instruction_mappings: HasMany<super::class_instruction_mappings::Entity>,
    #[sea_orm(has_many)]
    pub instruction_acknowledgements: HasMany<super::instruction_acknowledgements::Entity>,
    #[sea_orm(has_many)]
    pub safety_instruction_translations: HasMany<super::safety_instruction_translations::Entity>,
}

vespera::schema_type!(Schema from Model, name = "SafetyInstructionsSchema");
impl ActiveModelBehavior for ActiveModel {}
