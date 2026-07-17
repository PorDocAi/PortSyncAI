use sea_orm::entity::prelude::*;

/// 부서
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "departments")]
pub struct Model {
    /// 부서 고유번호
    #[sea_orm(primary_key)]
    pub department_id: i64,
    /// 부서 코드
    #[sea_orm(unique)]
    pub department_code: String,
    /// 부서명
    pub name: String,
    /// 설명
    pub description: Option<String>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(has_many)]
    pub employees: HasMany<super::employees::Entity>,
}

vespera::schema_type!(Schema from Model, name = "DepartmentsSchema");
impl ActiveModelBehavior for ActiveModel {}
