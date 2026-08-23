use sea_orm::entity::prelude::*;

/// 작업유형·위험물 등급별 필수 교육 규칙
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "education_target_rules")]
pub struct Model {
    /// 교육 대상 규칙 고유번호
    #[sea_orm(primary_key)]
    pub education_target_rule_id: i64,
    /// 교육 과정 FK
    #[sea_orm(indexed)]
    pub education_course_id: i64,
    /// 대상 작업 유형 FK
    pub work_type_id: Option<i64>,
    /// 대상 위험물 등급 FK
    pub dg_class_id: Option<i64>,
    /// 필수 교육 여부
    #[sea_orm(default_value = true)]
    pub is_required: bool,
    /// 규칙 사용 여부
    #[sea_orm(default_value = true)]
    pub is_active: bool,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "education_course_id", to = "education_course_id")]
    pub education_course: HasOne<super::education_courses::Entity>,
    #[sea_orm(belongs_to, from = "work_type_id", to = "work_type_id")]
    pub work_type: HasOne<super::work_types::Entity>,
    #[sea_orm(belongs_to, from = "dg_class_id", to = "dg_class_id")]
    pub dg_class: HasOne<super::dg_classes::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [education_course_id]
vespera::schema_type!(Schema from Model, name = "EducationTargetRulesSchema");
impl ActiveModelBehavior for ActiveModel {}
