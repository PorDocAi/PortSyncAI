use sea_orm::entity::prelude::*;

/// v2 장비 프로필 NFC 불투명 토큰 해시와 발급 수명주기
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "equipment_tag_tokens")]
pub struct Model {
    /// 장비 태그 토큰 고유번호
    #[sea_orm(primary_key)]
    pub equipment_tag_token_id: i64,
    /// v2 장비 프로필 FK
    #[sea_orm(indexed)]
    pub equipment_profile_id: i64,
    /// 서버 발급 불투명 토큰 SHA-256 해시
    #[sea_orm(unique)]
    pub tag_token_hash: String,
    /// 활성 토큰 여부
    #[sea_orm(indexed, default_value = true)]
    pub is_active: bool,
    /// 토큰 발급 관리자 FK
    pub issued_by_id: i64,
    /// 토큰 발급일시
    #[sea_orm(default_value = "NOW()")]
    pub issued_at: DateTimeWithTimeZone,
    /// 토큰 비활성화 관리자 FK
    pub deactivated_by_id: Option<i64>,
    /// 토큰 비활성화일시
    pub deactivated_at: Option<DateTimeWithTimeZone>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "equipment_profile_id", to = "equipment_profile_id")]
    pub equipment_profile: HasOne<super::equipment_profiles::Entity>,
    #[sea_orm(
        belongs_to,
        relation_enum = "IssuedBy",
        from = "issued_by_id",
        to = "employee_id"
    )]
    pub issued_by: HasOne<super::employees::Entity>,
    #[sea_orm(
        belongs_to,
        relation_enum = "DeactivatedBy",
        from = "deactivated_by_id",
        to = "employee_id"
    )]
    pub deactivated_by: HasOne<super::employees::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [equipment_profile_id]
// (unnamed) on [is_active]
vespera::schema_type!(Schema from Model, name = "EquipmentTagTokensSchema");
impl ActiveModelBehavior for ActiveModel {}
