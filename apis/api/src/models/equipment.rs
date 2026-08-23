use sea_orm::entity::prelude::*;

/// 개별 안전장비 (NFC 태그 단위, FR-D2)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "equipment")]
pub struct Model {
    /// 장비 고유번호
    #[sea_orm(primary_key)]
    pub equipment_id: i64,
    /// 장비 종류 FK
    #[sea_orm(indexed)]
    pub equipment_type_id: i64,
    /// 자산 관리번호
    #[sea_orm(unique)]
    pub asset_number: Option<String>,
    /// NFC 태그 UID (FR-D2)
    #[sea_orm(unique)]
    pub nfc_tag_uid: String,
    /// 점자 스티커 병기 내용 (FR-D2)
    pub braille_label: Option<String>,
    /// 사용 가능 여부
    #[sea_orm(default_value = true)]
    pub is_active: bool,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "equipment_type_id", to = "equipment_type_id")]
    pub equipment_type: HasOne<super::equipment_types::Entity>,
    #[sea_orm(has_many)]
    pub equipment_check_logs: HasMany<super::equipment_check_logs::Entity>,
    #[sea_orm(has_one)]
    pub equipment_profiles: HasOne<super::equipment_profiles::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [equipment_type_id]
vespera::schema_type!(Schema from Model, name = "EquipmentSchema");
impl ActiveModelBehavior for ActiveModel {}
