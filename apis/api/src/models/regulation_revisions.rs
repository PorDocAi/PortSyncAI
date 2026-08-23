use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "regulation_revisions_change_type")]
pub enum ChangeType {
    #[sea_orm(string_value = "ADDED")]
    Added,
    #[sea_orm(string_value = "MODIFIED")]
    Modified,
    #[sea_orm(string_value = "DELETED")]
    Deleted,
}

/// 신구조문대비표 개정 내역 (FR-A3, diff)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "regulation_revisions")]
pub struct Model {
    /// 개정내역 고유번호
    #[sea_orm(primary_key)]
    pub revision_id: i64,
    /// 출처 문서 FK
    #[sea_orm(indexed)]
    pub document_id: i64,
    /// 조문 번호
    pub clause_no: Option<String>,
    /// 변경 유형
    pub change_type: ChangeType,
    /// 구조문
    pub old_text: Option<String>,
    /// 신조문
    pub new_text: Option<String>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(belongs_to, from = "document_id", to = "document_id")]
    pub document: HasOne<super::regulation_documents::Entity>,
}

// Index definitions (SeaORM uses Statement builders externally)
// (unnamed) on [document_id]
vespera::schema_type!(Schema from Model, name = "RegulationRevisionsSchema");
impl ActiveModelBehavior for ActiveModel {}
