use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "equipment_profiles_equipment_ownership_type")]
pub enum EquipmentOwnershipType {
    #[sea_orm(string_value = "PERSONAL")]
    Personal,
    #[sea_orm(string_value = "SHARED")]
    Shared,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "equipment_profiles_equipment_lifecycle_status")]
pub enum EquipmentLifecycleStatus {
    #[sea_orm(string_value = "AVAILABLE")]
    Available,
    #[sea_orm(string_value = "BLOCKED")]
    Blocked,
    #[sea_orm(string_value = "DAMAGED")]
    Damaged,
    #[sea_orm(string_value = "LOST")]
    Lost,
    #[sea_orm(string_value = "REPLACED")]
    Replaced,
}

/// v2 소유정책·수명주기를 갖는 보호구 프로필
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "equipment_profiles")]
pub struct Model {
    /// v2 장비 프로필 고유번호
    #[sea_orm(primary_key)]
    pub equipment_profile_id: i64,
    /// 변환된 0001 장비 FK, v2 신규 장비는 null
    #[sea_orm(unique)]
    pub legacy_equipment_id: Option<i64>,
    /// 장비 종류 FK
    #[sea_orm(indexed)]
    pub equipment_type_id: i64,
    /// v2 자산 관리번호
    #[sea_orm(unique)]
    pub asset_number: Option<String>,
    /// 개인 또는 공용 소유정책
    #[sea_orm(indexed, default_value = "SHARED")]
    pub ownership_type: EquipmentOwnershipType,
    /// 개인 장비 소유자 FK, 공용은 null
    #[sea_orm(indexed)]
    pub owner_employee_id: Option<i64>,
    /// v2 장비 수명주기 상태
    #[sea_orm(indexed, default_value = "BLOCKED")]
    pub status: EquipmentLifecycleStatus,
    /// 상태 변경 사유
    pub status_reason: Option<String>,
    /// v2 점자 스티커 병기 내용
    pub braille_label: Option<String>,
    /// 해당 품목에만 적용하는 제조사 교체 권고일
    pub manufacturer_replacement_due_at: Option<Date>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "legacy_equipment_id", to = "equipment_id")]
    pub legacy: HasOne<super::equipment::Entity>,
    #[sea_orm(belongs_to, from = "equipment_type_id", to = "equipment_type_id")]
    pub equipment_type: HasOne<super::equipment_types::Entity>,
    #[sea_orm(belongs_to, from = "owner_employee_id", to = "employee_id")]
    pub owner: HasOne<super::employees::Entity>,
    #[sea_orm(has_many)]
    pub work_assignment_equipments: HasMany<super::work_assignment_equipment::Entity>,
    #[sea_orm(has_many)]
    pub equipment_tag_tokens: HasMany<super::equipment_tag_tokens::Entity>,
    #[sea_orm(has_many)]
    pub equipment_check_events: HasMany<super::equipment_check_events::Entity>,
    #[sea_orm(has_one)]
    pub shared_equipment_claims: HasOne<super::shared_equipment_claims::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [equipment_type_id]
// (unnamed) on [ownership_type]
// (unnamed) on [owner_employee_id]
// (unnamed) on [status]
vespera::schema_type!(Schema from Model, name = "EquipmentProfilesSchema");
impl ActiveModelBehavior for ActiveModel {}
