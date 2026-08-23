use sea_orm::entity::prelude::*;

/// 작업자 교육 이수 이력(재이수 허용)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "education_completions")]
pub struct Model {
    /// 교육 이수 고유번호
    #[sea_orm(primary_key)]
    pub education_completion_id: i64,
    /// 교육 과정 FK
    #[sea_orm(indexed)]
    pub education_course_id: i64,
    /// 이수 작업자 FK
    #[sea_orm(indexed)]
    pub employee_id: i64,
    /// 이수한 교육 과정 버전
    pub course_version: i32,
    /// 이수일시
    pub completed_at: DateTimeWithTimeZone,
    /// 이수 만료일시
    #[sea_orm(indexed)]
    pub expires_at: Option<DateTimeWithTimeZone>,
    /// 이수 증빙 경로
    pub evidence_url: Option<String>,
    /// 이수 등록자 FK
    pub recorded_by_id: i64,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "education_course_id", to = "education_course_id")]
    pub education_course: HasOne<super::education_courses::Entity>,
    #[sea_orm(
        belongs_to,
        relation_enum = "Employee",
        from = "employee_id",
        to = "employee_id"
    )]
    pub employee: HasOne<super::employees::Entity>,
    #[sea_orm(
        belongs_to,
        relation_enum = "RecordedBy",
        from = "recorded_by_id",
        to = "employee_id"
    )]
    pub recorded_by: HasOne<super::employees::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [education_course_id]
// (unnamed) on [employee_id]
// (unnamed) on [expires_at]
vespera::schema_type!(Schema from Model, name = "EducationCompletionsSchema");
impl ActiveModelBehavior for ActiveModel {}
