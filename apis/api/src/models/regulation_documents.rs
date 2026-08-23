use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "regulation_documents_source_law")]
pub enum SourceLaw {
    #[sea_orm(string_value = "OSH_ACT")]
    OshAct,
    #[sea_orm(string_value = "PORT_SAFETY_ACT")]
    PortSafetyAct,
    #[sea_orm(string_value = "KOSHA_GUIDE")]
    KoshaGuide,
    #[sea_orm(string_value = "IMDG")]
    Imdg,
    #[sea_orm(string_value = "OTHER")]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "regulation_documents_file_format")]
pub enum FileFormat {
    #[sea_orm(string_value = "HWP")]
    Hwp,
    #[sea_orm(string_value = "PDF")]
    Pdf,
    #[sea_orm(string_value = "XLSX")]
    Xlsx,
    #[sea_orm(string_value = "DOCX")]
    Docx,
    #[sea_orm(string_value = "IMAGE")]
    Image,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, vespera::Schema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "regulation_documents_processing_status")]
pub enum ProcessingStatus {
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "PROCESSING")]
    Processing,
    #[sea_orm(string_value = "COMPLETED")]
    Completed,
    #[sea_orm(string_value = "FAILED")]
    Failed,
}

/// 법령·공문 원문 업로드 (FR-A1)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "regulation_documents")]
pub struct Model {
    /// 문서 고유번호
    #[sea_orm(primary_key)]
    pub document_id: i64,
    /// 문서 제목
    pub title: String,
    /// 배포 기관 (관세청/해수부 등)
    pub issuing_authority: Option<String>,
    /// 근거 법령 (산안법/항만안전특별법/KOSHA/IMDG)
    pub source_law: SourceLaw,
    /// 파일 형식
    pub file_format: FileFormat,
    /// 원본 파일 경로
    pub file_url: String,
    /// 파일 해시 (무결성)
    pub file_hash: Option<String>,
    /// 일부개정 공문 여부 (FR-A3)
    #[sea_orm(default_value = false)]
    pub is_amendment: bool,
    /// 시행일
    pub effective_date: Option<Date>,
    /// 분석 처리 상태
    #[sea_orm(default_value = "PENDING")]
    pub processing_status: ProcessingStatus,
    /// 업로더 FK
    pub uploaded_by_id: i64,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "uploaded_by_id", to = "employee_id")]
    pub uploaded_by: HasOne<super::employees::Entity>,
    #[sea_orm(has_many)]
    pub regulation_revisions: HasMany<super::regulation_revisions::Entity>,
    #[sea_orm(has_many)]
    pub safety_instructions: HasMany<super::safety_instructions::Entity>,
}

vespera::schema_type!(Schema from Model, name = "RegulationDocumentsSchema");
impl ActiveModelBehavior for ActiveModel {}
