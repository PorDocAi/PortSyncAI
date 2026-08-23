use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "work_restriction_rules_restriction_condition")]
pub enum RestrictionCondition {
    #[sea_orm(string_value = "ASTHMA")]
    Asthma,
    #[sea_orm(string_value = "ALLERGY")]
    Allergy,
    #[sea_orm(string_value = "CLAUSTROPHOBIA")]
    Claustrophobia,
    #[sea_orm(string_value = "HEIGHT_LIMIT")]
    HeightLimit,
    #[sea_orm(string_value = "WEIGHT_LIMIT")]
    WeightLimit,
    #[sea_orm(string_value = "OTHER")]
    Other,
}

/// 투입 제한 규칙 (FR-F2, 설정 데이터 — 코드 수정 없이 교체, NFR 확장성)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "work_restriction_rules")]
pub struct Model {
    /// 규칙 고유번호
    #[sea_orm(primary_key)]
    pub rule_id: i64,
    /// 제한 조건 유형
    pub condition_type: RestrictionCondition,
    /// 조건 값 (예: 알레르겐 코드 RICE, 신장 임계값)
    pub condition_value: Option<String>,
    /// 제한 대상 위험물 등급 FK
    pub restricted_dg_class_id: Option<i64>,
    /// 제한 대상 작업유형 (등급으로 안잡히는 작업, 예: 밀폐공간)
    pub restricted_work_type: Option<String>,
    /// 설명
    pub description: Option<String>,
    /// 활성 여부
    #[sea_orm(default_value = true)]
    pub is_active: bool,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "restricted_dg_class_id", to = "dg_class_id")]
    pub restricted: HasOne<super::dg_classes::Entity>,
}

vespera::schema_type!(Schema from Model, name = "WorkRestrictionRulesSchema");
impl ActiveModelBehavior for ActiveModel {}
