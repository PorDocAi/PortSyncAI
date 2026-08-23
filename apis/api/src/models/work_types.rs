use sea_orm::entity::prelude::*;

/// 항만 작업 유형 기준정보
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "work_types")]
pub struct Model {
    /// 작업 유형 고유번호
    #[sea_orm(primary_key)]
    pub work_type_id: i64,
    /// 작업 유형 코드
    #[sea_orm(unique)]
    pub work_type_code: String,
    /// 작업 유형명
    pub name: String,
    /// 작업 유형 설명
    pub description: Option<String>,
    /// 사용 여부
    #[sea_orm(default_value = true)]
    pub is_active: bool,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(has_many)]
    pub works: HasMany<super::works::Entity>,
    #[sea_orm(has_many)]
    pub education_target_rules: HasMany<super::education_target_rules::Entity>,
}

vespera::schema_type!(Schema from Model, name = "WorkTypesSchema");
impl ActiveModelBehavior for ActiveModel {}
