//! v2 작업 배정 준비도와 gate_events 이중 기록 통합 테스트 (task-7v)

mod common;

use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};
use vespera::axum::http::StatusCode;

use api::models::{
    education_completions, education_courses, education_target_rules,
    equipment_profiles::EquipmentLifecycleStatus,
    work_assignment_equipment,
    work_stops::{self, WorkStopStatus},
};
use common::TestApp;

#[tokio::test]
async fn v2_assignment_with_all_sidecars_passes_and_appends_gate_event() {
    let app = common::spawn_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    let worker = common::seed_worker_with_started_attendance(&app.db).await;
    let equipment = common::seed_v2_equipment(
        &app.db,
        Some(worker.employee_id),
        api::models::equipment_profiles::EquipmentOwnershipType::Personal,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let assignment =
        common::seed_v2_assignment(&app.db, worker.employee_id, equipment.equipment_type_id).await;
    seed_course_completion(&app, worker.employee_id).await;
    allocate_snapshot(
        &app,
        &assignment.work_assignment_id,
        &equipment.equipment_profile_id,
    )
    .await;
    assert_eq!(gate_event_count(&app).await, 0);

    let request_body =
        request_with_assignment(common::WORKER_NFC_UID, assignment.work_assignment_id).await;
    let response = app
        .verify_gate_with(Some(&format!("Bearer {terminal_token}")), &request_body)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], true);
    assert_eq!(body["employee_name"], worker.name);
    assert_eq!(body["reason"], "통과");
    let event = latest_gate_event(&app).await;
    assert_eq!(event.decision, api::models::gate_events::GateDecision::Pass);
    assert_eq!(event.reason_code, "PASS");
    assert_eq!(
        event.work_assignment_id,
        Some(assignment.work_assignment_id)
    );
}

#[tokio::test]
async fn v2_assignment_without_required_allocation_is_blocked() {
    let app = common::spawn_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    let worker = common::seed_worker_with_started_attendance(&app.db).await;
    let equipment = common::seed_v2_equipment(
        &app.db,
        Some(worker.employee_id),
        api::models::equipment_profiles::EquipmentOwnershipType::Personal,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let assignment =
        common::seed_v2_assignment(&app.db, worker.employee_id, equipment.equipment_type_id).await;
    seed_course_completion(&app, worker.employee_id).await;
    assert_eq!(gate_event_count(&app).await, 0);

    let request_body =
        request_with_assignment(common::WORKER_NFC_UID, assignment.work_assignment_id).await;
    let response = app
        .verify_gate_with(Some(&format!("Bearer {terminal_token}")), &request_body)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], false);
    assert_eq!(body["reason"], "PPE_INCOMPLETE");
    let event = latest_gate_event(&app).await;
    assert_eq!(
        event.decision,
        api::models::gate_events::GateDecision::Block
    );
    assert_eq!(event.reason_code, "PPE_INCOMPLETE");
    assert_eq!(event.reason_codes.to_string(), r#"["PPE_INCOMPLETE"]"#);
}

#[tokio::test]
async fn v2_assignment_with_open_stop_is_blocked_and_appends_gate_event() {
    let app = common::spawn_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    let worker = common::seed_worker_with_started_attendance(&app.db).await;
    let equipment = common::seed_v2_equipment(
        &app.db,
        Some(worker.employee_id),
        api::models::equipment_profiles::EquipmentOwnershipType::Personal,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let assignment =
        common::seed_v2_assignment(&app.db, worker.employee_id, equipment.equipment_type_id).await;
    seed_course_completion(&app, worker.employee_id).await;
    allocate_snapshot(
        &app,
        &assignment.work_assignment_id,
        &equipment.equipment_profile_id,
    )
    .await;
    work_stops::ActiveModel {
        work_stop_id: Set(0),
        work_id: Set(Some(assignment.work_id)),
        work_assignment_id: Set(Some(assignment.work_assignment_id)),
        legacy_attendance_id: Set(None),
        status: Set(WorkStopStatus::Open),
        reason: Set("안전 정지".to_string()),
        stopped_by_id: Set(Some(worker.employee_id)),
        stopped_at: Set(chrono::Utc::now().fixed_offset()),
        closed_by_id: Set(None),
        closed_at: Set(None),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        updated_at: Set(None),
    }
    .insert(&app.db)
    .await
    .expect("작업 중지 픽스처 삽입 실패");
    assert_eq!(gate_event_count(&app).await, 0);

    let response = app
        .verify_gate_with(
            Some(&format!("Bearer {terminal_token}")),
            &request_with_assignment(common::WORKER_NFC_UID, assignment.work_assignment_id).await,
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], false);
    assert_eq!(body["reason"], "WORK_STOPPED");
    let event = latest_gate_event(&app).await;
    assert_eq!(
        event.decision,
        api::models::gate_events::GateDecision::Block
    );
    assert_eq!(event.reason_code, "WORK_STOPPED");
}

async fn request_with_assignment(nfc_card_uid: &str, work_assignment_id: i64) -> String {
    serde_json::json!({
        "nfc_card_uid": nfc_card_uid,
        "v2_work_assignment_id": work_assignment_id,
    })
    .to_string()
}

async fn allocate_snapshot(app: &TestApp, assignment_id: &i64, profile_id: &i64) {
    let work_id = assignment_work_id(app, assignment_id).await;
    let snapshot_id: i64 = app
        .db
        .query_one_raw(sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            "SELECT work_ppe_requirement_snapshot_id FROM work_ppe_requirement_snapshots WHERE work_id = ? LIMIT 1",
            vec![sea_orm::Value::BigInt(Some(work_id))],
        ))
        .await
        .expect("스냅샷 조회 실패")
        .expect("PPE 스냅샷이 존재해야 한다")
        .try_get("", "work_ppe_requirement_snapshot_id")
        .expect("스냅샷 ID 조회 실패");
    work_assignment_equipment::ActiveModel {
        work_assignment_equipment_id: Set(0),
        work_assignment_id: Set(*assignment_id),
        equipment_profile_id: Set(*profile_id),
        requirement_snapshot_id: Set(snapshot_id),
        accepted_at: Set(chrono::Utc::now().fixed_offset()),
    }
    .insert(&app.db)
    .await
    .expect("장비 할당 픽스처 삽입 실패");
}

async fn assignment_work_id(app: &TestApp, assignment_id: &i64) -> i64 {
    api::models::v2_work_assignments::Entity::find_by_id(*assignment_id)
        .one(&app.db)
        .await
        .expect("배정 조회 실패")
        .expect("v2 배정이 존재해야 한다")
        .work_id
}

async fn seed_course_completion(app: &TestApp, employee_id: i64) {
    let course = education_courses::ActiveModel {
        education_course_id: Set(0),
        course_code: Set(format!("EDU-{}", uuid::Uuid::new_v4().simple())),
        title: Set("필수 안전교육".to_string()),
        description: Set(None),
        version: Set(1),
        validity_days: Set(None),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        updated_at: Set(None),
    }
    .insert(&app.db)
    .await
    .expect("교육 과정 픽스처 삽입 실패");
    education_target_rules::ActiveModel {
        education_target_rule_id: Set(0),
        education_course_id: Set(course.education_course_id),
        work_type_id: Set(Some(assignment_work_type_id(app).await)),
        dg_class_id: Set(None),
        is_required: Set(true),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        updated_at: Set(None),
    }
    .insert(&app.db)
    .await
    .expect("교육 대상 규칙 픽스처 삽입 실패");
    education_completions::ActiveModel {
        education_completion_id: Set(0),
        education_course_id: Set(course.education_course_id),
        employee_id: Set(employee_id),
        course_version: Set(1),
        completed_at: Set(chrono::Utc::now().fixed_offset()),
        expires_at: Set(None),
        evidence_url: Set(None),
        recorded_by_id: Set(employee_id),
        created_at: Set(chrono::Utc::now().fixed_offset()),
    }
    .insert(&app.db)
    .await
    .expect("교육 이수 픽스처 삽입 실패");
}

async fn assignment_work_type_id(app: &TestApp) -> i64 {
    api::models::works::Entity::find()
        .one(&app.db)
        .await
        .expect("작업 조회 실패")
        .expect("v2 작업이 존재해야 한다")
        .work_type_id
}

async fn gate_event_count(app: &TestApp) -> usize {
    api::models::gate_events::Entity::find()
        .all(&app.db)
        .await
        .expect("게이트 이벤트 조회 실패")
        .len()
}

async fn latest_gate_event(app: &TestApp) -> api::models::gate_events::Model {
    use sea_orm::QueryOrder;
    api::models::gate_events::Entity::find()
        .order_by_desc(api::models::gate_events::Column::GateEventId)
        .one(&app.db)
        .await
        .expect("게이트 이벤트 조회 실패")
        .expect("게이트 이벤트가 존재해야 한다")
}
