use sea_orm::entity::prelude::*;

/// 직무
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "job_roles")]
pub struct Model {
    /// 직무 고유번호
    #[sea_orm(primary_key)]
    pub job_role_id: i64,
    /// 직무 코드
    #[sea_orm(unique)]
    pub job_role_code: String,
    /// 직무명 (하역/검수/컨테이너점검 등)
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

vespera::schema_type!(Schema from Model, name = "JobRolesSchema");
impl ActiveModelBehavior for ActiveModel {}
