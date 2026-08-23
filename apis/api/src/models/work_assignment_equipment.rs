use sea_orm::entity::prelude::*;

/// v2 작업 배정에 수락된 보호구 할당 이력
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "work_assignment_equipment")]
pub struct Model {
    /// 작업 장비 할당 고유번호
    #[sea_orm(primary_key)]
    pub work_assignment_equipment_id: i64,
    /// v2 작업 배정 FK
    pub work_assignment_id: i64,
    /// 수락 v2 장비 프로필 FK
    pub equipment_profile_id: i64,
    /// 충족한 보호구 스냅샷 FK
    pub requirement_snapshot_id: i64,
    /// 수락일시
    #[sea_orm(default_value = "NOW()")]
    pub accepted_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "work_assignment_id", to = "work_assignment_id")]
    pub work_assignment: HasOne<super::v2_work_assignments::Entity>,
    #[sea_orm(belongs_to, from = "equipment_profile_id", to = "equipment_profile_id")]
    pub equipment_profile: HasOne<super::equipment_profiles::Entity>,
    #[sea_orm(
        belongs_to,
        from = "requirement_snapshot_id",
        to = "work_ppe_requirement_snapshot_id"
    )]
    pub requirement_snapshot: HasOne<super::work_ppe_requirement_snapshots::Entity>,
}

/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["work_assignment_id", "equipment_profile_id"], // uq_assignment_equipment
];
vespera::schema_type!(Schema from Model, name = "WorkAssignmentEquipmentSchema");
impl ActiveModelBehavior for ActiveModel {}
