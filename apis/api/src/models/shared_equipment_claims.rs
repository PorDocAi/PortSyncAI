use sea_orm::entity::prelude::*;

/// v2 공용 장비 프로필의 활성 작업 배타적 점유
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "shared_equipment_claims")]
pub struct Model {
    /// 공용 장비 점유 고유번호
    #[sea_orm(primary_key)]
    pub shared_equipment_claim_id: i64,
    /// v2 점유 작업 배정 FK
    pub work_assignment_id: i64,
    /// 점유 v2 공용 장비 프로필 FK
    #[sea_orm(unique)]
    pub equipment_profile_id: i64,
    /// 점유 시작일시
    #[sea_orm(default_value = "NOW()")]
    pub claimed_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "work_assignment_id", to = "work_assignment_id")]
    pub work_assignment: HasOne<super::v2_work_assignments::Entity>,
    #[sea_orm(belongs_to, from = "equipment_profile_id", to = "equipment_profile_id")]
    pub equipment_profile: HasOne<super::equipment_profiles::Entity>,
}

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["work_assignment_id", "equipment_profile_id"], // uq_shared_equipment_assignment
];
vespera::schema_type!(Schema from Model, name = "SharedEquipmentClaimsSchema");
impl ActiveModelBehavior for ActiveModel {}
