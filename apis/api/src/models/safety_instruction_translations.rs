use sea_orm::entity::prelude::*;

/// 안전지침 다국어/TTS/점자 변환 (FR-A4, C1, NFR 접근성)
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "safety_instruction_translations")]
pub struct Model {
    /// 번역 고유번호
    #[sea_orm(primary_key)]
    pub translation_id: i64,
    /// 안전지침 FK
    pub instruction_id: i64,
    /// 언어 코드 (ko/en/vi/zh 등)
    pub language_code: String,
    /// 번역 제목
    pub translated_title: String,
    /// 번역 본문
    pub translated_body: String,
    /// TTS 음성 파일 경로
    pub tts_audio_url: Option<String>,
    /// 점자 변환 데이터 경로
    pub braille_data_url: Option<String>,
    /// 생성일시
    #[sea_orm(default_value = "NOW()")]
    pub created_at: DateTimeWithTimeZone,
    /// 수정일시
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(belongs_to, from = "instruction_id", to = "instruction_id")]
    pub instruction: HasOne<super::safety_instructions::Entity>,
}


/// Composite unique constraints — declare in migrations or use Statement builder.
pub const COMPOSITE_UNIQUES: &[&[&str]] = &[
    &["instruction_id", "language_code"], // uq_instruction_language
];
vespera::schema_type!(Schema from Model, name = "SafetyInstructionTranslationsSchema");
impl ActiveModelBehavior for ActiveModel {}
