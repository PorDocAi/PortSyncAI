use sea_orm::entity::prelude::*;

/// 작업 투입 전 교육 과정 기준정보
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "education_courses")]
pub struct Model {
    /// 교육 과정 고유번호
    #[sea_orm(primary_key)]
    pub education_course_id: i64,
    /// 교육 과정 코드
    #[sea_orm(unique)]
    pub course_code: String,
    /// 교육 과정명
    pub title: String,
    /// 교육 과정 설명
    pub description: Option<String>,
    /// 교육 과정 버전
    #[sea_orm(default_value = 1)]
    pub version: i32,
    /// 이수 유효기간 일수, null이면 무기한
    pub validity_days: Option<i32>,
    /// 사용 여부
    #[sea_orm(default_value = true)]
    pub is_active: bool,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(has_many)]
    pub education_completions: HasMany<super::education_completions::Entity>,
    #[sea_orm(has_many)]
    pub education_target_rules: HasMany<super::education_target_rules::Entity>,
}

vespera::schema_type!(Schema from Model, name = "EducationCoursesSchema");
impl ActiveModelBehavior for ActiveModel {}
