use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::Deserialize;

use crate::models::class_equipment_mappings::{self, RequirementLevel};
use crate::models::safety_instructions::SourceLaw;
use crate::models::{class_instruction_mappings, dg_classes, equipment_types, safety_instructions};

/// seeds/seed_master_data.json 을 컴파일 타임에 임베드 (실행 위치 무관)
const SEED_JSON: &str = include_str!("../../seeds/seed_master_data.json");

#[derive(Deserialize)]
struct SeedData {
    dg_classes: Vec<DgClassSeed>,
    equipment_types: Vec<EquipmentTypeSeed>,
    class_equipment_mappings: Vec<MappingSeed>,
    #[serde(default)]
    safety_instructions: Vec<SafetyInstructionSeed>,
    #[serde(default)]
    class_instruction_mappings: Vec<InstructionMappingSeed>,
}

#[derive(Deserialize)]
struct DgClassSeed {
    dg_class_id: i64,
    class_code: String,
    name_ko: String,
    name_en: Option<String>,
    description: Option<String>,
    imdg_version: Option<String>,
}

#[derive(Deserialize)]
struct EquipmentTypeSeed {
    equipment_type_id: i64,
    name: String,
    category: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct MappingSeed {
    dg_class_id: i64,
    equipment_type_id: i64,
    requirement_level: String,
    imdg_version: Option<String>,
}

#[derive(Deserialize)]
struct SafetyInstructionSeed {
    instruction_id: i64,
    title: String,
    summary: String,
    source_law: String,
    version: i32,
}

#[derive(Deserialize)]
struct InstructionMappingSeed {
    dg_class_id: i64,
    instruction_id: i64,
}

/// 마스터 데이터(위험물 등급·장비·매핑) 멱등 적재.
/// 앱 시작 시 호출하며, 이미 존재하는 행은 건너뛴다.
pub async fn load_master_data(db: &DatabaseConnection) {
    let data: SeedData = serde_json::from_str(SEED_JSON).expect("시드 JSON 파싱 실패");

    for c in data.dg_classes {
        let exists = dg_classes::Entity::find_by_id(c.dg_class_id)
            .one(db)
            .await
            .expect("dg_class 조회 실패")
            .is_some();
        if !exists {
            dg_classes::ActiveModel {
                dg_class_id: Set(c.dg_class_id),
                class_code: Set(c.class_code),
                name_ko: Set(c.name_ko),
                name_en: Set(c.name_en),
                description: Set(c.description),
                imdg_version: Set(c.imdg_version),
                ..Default::default()
            }
            .insert(db)
            .await
            .expect("dg_class 시드 삽입 실패");
        }
    }

    for e in data.equipment_types {
        let exists = equipment_types::Entity::find_by_id(e.equipment_type_id)
            .one(db)
            .await
            .expect("equipment_type 조회 실패")
            .is_some();
        if !exists {
            equipment_types::ActiveModel {
                equipment_type_id: Set(e.equipment_type_id),
                name: Set(e.name),
                category: Set(e.category),
                description: Set(e.description),
                ..Default::default()
            }
            .insert(db)
            .await
            .expect("equipment_type 시드 삽입 실패");
        }
    }

    for m in data.class_equipment_mappings {
        let exists = class_equipment_mappings::Entity::find()
            .filter(class_equipment_mappings::Column::DgClassId.eq(m.dg_class_id))
            .filter(class_equipment_mappings::Column::EquipmentTypeId.eq(m.equipment_type_id))
            .one(db)
            .await
            .expect("매핑 조회 실패")
            .is_some();
        if !exists {
            let level = match m.requirement_level.as_str() {
                "RECOMMENDED" => RequirementLevel::Recommended,
                _ => RequirementLevel::Required,
            };
            class_equipment_mappings::ActiveModel {
                dg_class_id: Set(m.dg_class_id),
                equipment_type_id: Set(m.equipment_type_id),
                requirement_level: Set(level),
                imdg_version: Set(m.imdg_version),
                ..Default::default()
            }
            .insert(db)
            .await
            .expect("매핑 시드 삽입 실패");
        }
    }

    for instruction in data.safety_instructions {
        let exists = safety_instructions::Entity::find_by_id(instruction.instruction_id)
            .one(db)
            .await
            .expect("안전지침 조회 실패")
            .is_some();
        if !exists {
            let source_law = match instruction.source_law.as_str() {
                "OSH_ACT" => SourceLaw::OshAct,
                "PORT_SAFETY_ACT" => SourceLaw::PortSafetyAct,
                "KOSHA_GUIDE" => SourceLaw::KoshaGuide,
                "IMDG" => SourceLaw::Imdg,
                _ => SourceLaw::Other,
            };
            safety_instructions::ActiveModel {
                instruction_id: Set(instruction.instruction_id),
                title: Set(instruction.title),
                summary: Set(instruction.summary),
                source_law: Set(source_law),
                version: Set(instruction.version),
                is_active: Set(true),
                ..Default::default()
            }
            .insert(db)
            .await
            .expect("안전지침 시드 삽입 실패");
        }
    }

    for mapping in data.class_instruction_mappings {
        let exists = class_instruction_mappings::Entity::find()
            .filter(class_instruction_mappings::Column::DgClassId.eq(mapping.dg_class_id))
            .filter(class_instruction_mappings::Column::InstructionId.eq(mapping.instruction_id))
            .one(db)
            .await
            .expect("Class-지침 매핑 조회 실패")
            .is_some();
        if !exists {
            class_instruction_mappings::ActiveModel {
                dg_class_id: Set(mapping.dg_class_id),
                instruction_id: Set(mapping.instruction_id),
                ..Default::default()
            }
            .insert(db)
            .await
            .expect("Class-지침 매핑 시드 삽입 실패");
        }
    }
}
