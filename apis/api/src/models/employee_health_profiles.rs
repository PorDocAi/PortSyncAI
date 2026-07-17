use sea_orm::entity::prelude::*;

/// 직원 건강·신체 정보 (FR-F1, 민감정보 분리 저장)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "employee_health_profiles")]
pub struct Model {
    /// 건강정보 고유번호
    #[sea_orm(primary_key)]
    pub health_profile_id: i64,
    /// 직원 FK (1:1)
    #[sea_orm(unique)]
    pub employee_id: i64,
    /// 신장(cm)
    pub height_cm: Option<i32>,
    /// 체중(kg)
    pub weight_kg: Option<i32>,
    /// 천식 보유 여부
    #[sea_orm(default_value = false)]
    pub has_asthma: bool,
    /// 폐쇄공포 보유 여부
    #[sea_orm(default_value = false)]
    pub has_claustrophobia: bool,
    /// 알레르기 유발물질 코드 배열 (예: ["RICE"])
    pub allergens: Option<Json>,
    /// 기타 특이사항
    pub other_conditions: Option<String>,
    /// 개인정보(건강) 수집·이용 동의
    #[sea_orm(default_value = false)]
    pub consent_agreed: bool,
    /// 동의 시각
    pub consent_at: Option<DateTimeWithTimeZone>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "employee_id", to = "employee_id")]
    pub employee: HasOne<super::employees::Entity>,
}

vespera::schema_type!(Schema from Model, name = "EmployeeHealthProfilesSchema");
impl ActiveModelBehavior for ActiveModel {}
