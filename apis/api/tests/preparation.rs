//! v2 preparation sidecar integration coverage.
//!
//! The fixtures use only sidecar work/assignment, education, PPE, preparation,
//! and stop rows. The legacy attendance row is read before/after to ensure the
//! preparation APIs do not use the global equipment boolean.

mod common;

use api::models::{
    attendances,
    cargo_document_versions::{
        self, CargoProcessingStatus, CargoReviewStatus, V2CargoDocumentType,
    },
    employees, v2_work_assignments, work_types, works,
};
use api::routes::{education, ppe, work_preparations, work_stops};
use chrono::{DateTime, Utc};
use common::TestApp;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, EntityTrait, Set};
use vespera::axum::{
    http::{Method, StatusCode},
    routing::{get, post},
};

struct PreparationFixture {
    app: TestApp,
    worker: employees::Model,
    attendance: attendances::Model,
    work: works::Model,
    assignment: v2_work_assignments::Model,
    document_version: cargo_document_versions::Model,
    equipment_type_id: i64,
}

async fn spawn_preparation_app() -> TestApp {
    common::spawn_app_with(|router| {
        router
            .route(
                "/education/courses",
                get(education::list_education_courses).post(education::create_education_course),
            )
            .route(
                "/education/rules",
                get(education::list_education_target_rules)
                    .post(education::create_education_target_rule),
            )
            .route(
                "/education/completions",
                get(education::list_education_completions)
                    .post(education::create_education_completion),
            )
            .route(
                "/education/readiness",
                get(education::get_education_readiness),
            )
            .route(
                "/ppe/requirements",
                get(ppe::list_ppe_requirements).post(ppe::create_ppe_requirement),
            )
            .route(
                "/ppe/requirements/{id}/confirm",
                post(ppe::confirm_ppe_requirement),
            )
            .route(
                "/ppe/snapshots",
                get(ppe::list_ppe_snapshots).post(ppe::create_ppe_snapshot),
            )
            .route(
                "/work-preparations",
                post(work_preparations::upsert_work_preparation),
            )
            .route(
                "/work-preparations/{id}",
                get(work_preparations::get_work_preparation),
            )
            .route(
                "/work-stops",
                get(work_stops::list_work_stops).post(work_stops::create_work_stop),
            )
            .route("/work-stops/{id}/close", post(work_stops::close_work_stop))
    })
    .await
}

async fn fixture() -> PreparationFixture {
    let app = spawn_preparation_app().await;
    let (worker, attendance) = common::spawn_worker(&app.db, "preparation-worker").await;
    let equipment = common::seed_active_equipment(&app.db).await;
    let now: DateTime<chrono::FixedOffset> = Utc::now().into();

    let work_type = work_types::ActiveModel {
        work_type_id: NotSet,
        work_type_code: Set(format!("PREP-{}", uuid::Uuid::new_v4().simple())),
        name: Set("컨테이너 작업".to_string()),
        description: Set(Some("preparation test".to_string())),
        is_active: Set(true),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(&app.db)
    .await
    .expect("작업 유형 픽스처 삽입 실패");

    let work = works::ActiveModel {
        work_id: NotSet,
        work_type_id: Set(work_type.work_type_id),
        work_reference: Set(format!("WORK-{}", uuid::Uuid::new_v4().simple())),
        status: Default::default(),
        scheduled_start_at: Set(now),
        scheduled_end_at: Set(None),
        started_at: Set(None),
        completed_at: Set(None),
        created_by_id: Set(worker.employee_id),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(&app.db)
    .await
    .expect("작업 픽스처 삽입 실패");

    let assignment = v2_work_assignments::ActiveModel {
        work_assignment_id: NotSet,
        legacy_assignment_id: Set(None),
        work_id: Set(work.work_id),
        employee_id: Set(worker.employee_id),
        assigned_by_id: Set(worker.employee_id),
        status: Default::default(),
        selected_at: Set(None),
        started_at: Set(None),
        completed_at: Set(None),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(&app.db)
    .await
    .expect("v2 작업 배정 픽스처 삽입 실패");

    let document_version = cargo_document_versions::ActiveModel {
        cargo_document_version_id: NotSet,
        legacy_cargo_document_id: Set(None),
        document_type: Set(V2CargoDocumentType::Msds),
        document_version: Set(3),
        processing_status: Set(CargoProcessingStatus::Completed),
        review_status: Set(CargoReviewStatus::Confirmed),
        reviewed_by_id: Set(Some(worker.employee_id)),
        reviewed_at: Set(Some(now)),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(&app.db)
    .await
    .expect("문서 버전 픽스처 삽입 실패");

    PreparationFixture {
        app,
        worker,
        attendance,
        work,
        assignment,
        document_version,
        equipment_type_id: equipment.equipment_type_id,
    }
}

fn request(
    method: Method,
    uri: &str,
    authorization: &str,
    body: serde_json::Value,
) -> vespera::axum::http::Request<String> {
    vespera::axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", authorization)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .expect("요청 생성 실패")
}

async fn send(
    app: &TestApp,
    method: Method,
    uri: &str,
    authorization: &str,
    body: serde_json::Value,
) -> common::MockResponse {
    app.send_request(request(method, uri, authorization, body))
        .await
}

fn admin_bearer_for(employee_id: i64) -> String {
    common::bearer_for(employee_id, "ADMIN")
}

fn worker_bearer(employee_id: i64) -> String {
    common::bearer_for(employee_id, "WORKER")
}

#[tokio::test]
async fn education_readiness_distinguishes_missing_expired_and_current() {
    let fixture = fixture().await;
    let admin = admin_bearer_for(fixture.worker.employee_id);
    let worker = worker_bearer(fixture.worker.employee_id);

    let course = send(
        &fixture.app,
        Method::POST,
        "/education/courses",
        &admin,
        serde_json::json!({
            "course_code": "EDU-PREP-001",
            "title": "작업 전 안전교육",
            "validity_days": 30
        }),
    )
    .await;
    assert_eq!(course.status(), StatusCode::CREATED);
    let course_id = course.json()["education_course_id"]
        .as_i64()
        .expect("교육 과정 ID 없음");

    let rule = send(
        &fixture.app,
        Method::POST,
        "/education/rules",
        &admin,
        serde_json::json!({
            "education_course_id": course_id,
            "work_type_id": fixture.work.work_type_id,
            "is_required": true
        }),
    )
    .await;
    assert_eq!(rule.status(), StatusCode::CREATED);

    let courses = send(
        &fixture.app,
        Method::GET,
        "/education/courses",
        &worker,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(courses.status(), StatusCode::OK);
    assert_eq!(courses.json().as_array().unwrap().len(), 1);
    let rules = send(
        &fixture.app,
        Method::GET,
        "/education/rules",
        &worker,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(rules.status(), StatusCode::OK);
    assert_eq!(rules.json().as_array().unwrap().len(), 1);

    let missing = send(
        &fixture.app,
        Method::GET,
        &format!(
            "/education/readiness?employee_id={}&work_id={}",
            fixture.worker.employee_id, fixture.work.work_id
        ),
        &worker,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(missing.status(), StatusCode::OK);
    assert!(!missing.json()["fulfilled"].as_bool().unwrap());
    assert_eq!(missing.json()["requirements"][0]["status"], "MISSING");

    let expired = send(
        &fixture.app,
        Method::POST,
        "/education/completions",
        &admin,
        serde_json::json!({
            "education_course_id": course_id,
            "employee_id": fixture.worker.employee_id,
            "completed_at": "2020-01-01T00:00:00Z",
            "expires_at": "2020-02-01T00:00:00Z"
        }),
    )
    .await;
    assert_eq!(expired.status(), StatusCode::CREATED);

    let expired_readiness = send(
        &fixture.app,
        Method::GET,
        &format!(
            "/education/readiness?employee_id={}&work_id={}",
            fixture.worker.employee_id, fixture.work.work_id
        ),
        &worker,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(
        expired_readiness.json()["requirements"][0]["status"],
        "EXPIRED"
    );
    assert!(!expired_readiness.json()["fulfilled"].as_bool().unwrap());

    let current = send(
        &fixture.app,
        Method::POST,
        "/education/completions",
        &admin,
        serde_json::json!({
            "education_course_id": course_id,
            "employee_id": fixture.worker.employee_id,
            "completed_at": "2026-01-01T00:00:00Z",
            "expires_at": "2099-01-01T00:00:00Z"
        }),
    )
    .await;
    assert_eq!(current.status(), StatusCode::CREATED);
    assert!(current.json()["fulfilled"].as_bool().unwrap());

    let completions = send(
        &fixture.app,
        Method::GET,
        &format!(
            "/education/completions?employee_id={}",
            fixture.worker.employee_id
        ),
        &worker,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(completions.status(), StatusCode::OK);
    assert_eq!(completions.json().as_array().unwrap().len(), 2);

    let current_readiness = send(
        &fixture.app,
        Method::GET,
        &format!(
            "/education/readiness?employee_id={}&work_id={}",
            fixture.worker.employee_id, fixture.work.work_id
        ),
        &worker,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(
        current_readiness.json()["requirements"][0]["status"],
        "CURRENT"
    );
    assert!(current_readiness.json()["fulfilled"].as_bool().unwrap());
}

#[tokio::test]
async fn confirmed_ppe_requirement_can_be_snapshotted_and_queried() {
    let fixture = fixture().await;
    let admin = admin_bearer_for(fixture.worker.employee_id);
    let worker = worker_bearer(fixture.worker.employee_id);

    let requirement = send(
        &fixture.app,
        Method::POST,
        "/ppe/requirements",
        &admin,
        serde_json::json!({
            "equipment_type_id": fixture.equipment_type_id,
            "category": "HAND_PROTECTION",
            "performance_criteria": {"standard": "EN 374"},
            "source_text": "화학물질 취급 시 내화학 장갑",
            "source_document_version_id": fixture.document_version.cargo_document_version_id
        }),
    )
    .await;
    assert_eq!(requirement.status(), StatusCode::CREATED);
    assert_eq!(requirement.json()["review_status"], "PENDING");
    let requirement_id = requirement.json()["ppe_requirement_id"]
        .as_i64()
        .expect("PPE 요구조건 ID 없음");

    let confirmed = send(
        &fixture.app,
        Method::POST,
        &format!("/ppe/requirements/{requirement_id}/confirm"),
        &admin,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(confirmed.status(), StatusCode::OK);
    assert_eq!(confirmed.json()["review_status"], "CONFIRMED");

    let snapshot = send(
        &fixture.app,
        Method::POST,
        "/ppe/snapshots",
        &admin,
        serde_json::json!({
            "work_id": fixture.work.work_id,
            "ppe_requirement_id": requirement_id
        }),
    )
    .await;
    assert_eq!(snapshot.status(), StatusCode::CREATED);
    assert_eq!(snapshot.json()["category"], "HAND_PROTECTION");
    assert_eq!(snapshot.json()["source_document_version_number"], 3);

    let queried = send(
        &fixture.app,
        Method::GET,
        &format!("/ppe/snapshots?work_id={}", fixture.work.work_id),
        &worker,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(queried.status(), StatusCode::OK);
    assert_eq!(queried.json().as_array().unwrap().len(), 1);
    assert_eq!(
        queried.json()[0]["work_ppe_requirement_snapshot_id"],
        snapshot.json()["work_ppe_requirement_snapshot_id"]
    );

    let attendance_after = attendances::Entity::find_by_id(fixture.attendance.attendance_id)
        .one(&fixture.app.db)
        .await
        .expect("출근 조회 실패")
        .expect("출근 행 없음");
    assert_eq!(
        attendance_after.equipment_check_completed, fixture.attendance.equipment_check_completed,
        "PPE sidecar API가 전역 출근 장비 boolean을 변경하면 안 된다"
    );
}

#[tokio::test]
async fn work_stop_round_trips_open_and_closed_state() {
    let fixture = fixture().await;
    let admin = admin_bearer_for(fixture.worker.employee_id);
    let worker = worker_bearer(fixture.worker.employee_id);

    let opened = send(
        &fixture.app,
        Method::POST,
        "/work-stops",
        &admin,
        serde_json::json!({
            "work_assignment_id": fixture.assignment.work_assignment_id,
            "reason": "강풍으로 작업을 일시 중지합니다."
        }),
    )
    .await;
    assert_eq!(opened.status(), StatusCode::CREATED);
    assert_eq!(opened.json()["status"], "OPEN");
    assert!(!opened.json()["fulfilled"].as_bool().unwrap());
    let stop_id = opened.json()["work_stop_id"].as_i64().unwrap();

    let open_query = send(
        &fixture.app,
        Method::GET,
        &format!(
            "/work-stops?work_assignment_id={}",
            fixture.assignment.work_assignment_id
        ),
        &worker,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(open_query.status(), StatusCode::OK);
    assert_eq!(open_query.json()[0]["status"], "OPEN");

    let closed = send(
        &fixture.app,
        Method::POST,
        &format!("/work-stops/{stop_id}/close"),
        &admin,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(closed.status(), StatusCode::OK);
    assert_eq!(closed.json()["status"], "CLOSED");
    assert!(closed.json()["fulfilled"].as_bool().unwrap());

    let closed_query = send(
        &fixture.app,
        Method::GET,
        &format!(
            "/work-stops?work_assignment_id={}",
            fixture.assignment.work_assignment_id
        ),
        &worker,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(closed_query.json()[0]["status"], "CLOSED");
    assert!(closed_query.json()[0]["fulfilled"].as_bool().unwrap());
}

#[tokio::test]
async fn preparation_is_blocked_by_open_stop_and_ready_after_close() {
    let fixture = fixture().await;
    let admin = admin_bearer_for(fixture.worker.employee_id);

    let course = send(
        &fixture.app,
        Method::POST,
        "/education/courses",
        &admin,
        serde_json::json!({
            "course_code": "EDU-PREP-READY",
            "title": "준비 판정 교육",
            "validity_days": 3650
        }),
    )
    .await;
    let course_id = course.json()["education_course_id"].as_i64().unwrap();
    let rule = send(
        &fixture.app,
        Method::POST,
        "/education/rules",
        &admin,
        serde_json::json!({
            "education_course_id": course_id,
            "work_type_id": fixture.work.work_type_id
        }),
    )
    .await;
    assert_eq!(rule.status(), StatusCode::CREATED);
    let completion = send(
        &fixture.app,
        Method::POST,
        "/education/completions",
        &admin,
        serde_json::json!({
            "education_course_id": course_id,
            "employee_id": fixture.worker.employee_id,
            "completed_at": "2026-01-01T00:00:00Z",
            "expires_at": "2099-01-01T00:00:00Z"
        }),
    )
    .await;
    assert_eq!(completion.status(), StatusCode::CREATED);

    let requirement = send(
        &fixture.app,
        Method::POST,
        "/ppe/requirements",
        &admin,
        serde_json::json!({
            "equipment_type_id": fixture.equipment_type_id,
            "category": "EYE_PROTECTION",
            "source_text": "보안경",
            "source_document_version_id": fixture.document_version.cargo_document_version_id
        }),
    )
    .await;
    assert_eq!(requirement.status(), StatusCode::CREATED);
    let requirement_id = requirement.json()["ppe_requirement_id"].as_i64().unwrap();
    let confirmed = send(
        &fixture.app,
        Method::POST,
        &format!("/ppe/requirements/{requirement_id}/confirm"),
        &admin,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(confirmed.status(), StatusCode::OK);
    let snapshot = send(
        &fixture.app,
        Method::POST,
        "/ppe/snapshots",
        &admin,
        serde_json::json!({
            "work_id": fixture.work.work_id,
            "ppe_requirement_id": requirement_id
        }),
    )
    .await;
    assert_eq!(snapshot.status(), StatusCode::CREATED);

    let opened = send(
        &fixture.app,
        Method::POST,
        "/work-stops",
        &admin,
        serde_json::json!({
            "work_assignment_id": fixture.assignment.work_assignment_id,
            "reason": "작업구역 점검"
        }),
    )
    .await;
    let stop_id = opened.json()["work_stop_id"].as_i64().unwrap();

    let blocked = send(
        &fixture.app,
        Method::POST,
        "/work-preparations",
        &admin,
        serde_json::json!({
            "work_assignment_id": fixture.assignment.work_assignment_id,
            "document_ready": true,
            "instruction_ready": true
        }),
    )
    .await;
    assert_eq!(blocked.status(), StatusCode::CREATED);
    assert_eq!(blocked.json()["status"], "BLOCKED");
    assert!(!blocked.json()["fulfilled"].as_bool().unwrap());

    let closed = send(
        &fixture.app,
        Method::POST,
        &format!("/work-stops/{stop_id}/close"),
        &admin,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(closed.status(), StatusCode::OK);

    let ready = send(
        &fixture.app,
        Method::POST,
        "/work-preparations",
        &admin,
        serde_json::json!({
            "work_assignment_id": fixture.assignment.work_assignment_id,
            "document_ready": true,
            "instruction_ready": true
        }),
    )
    .await;
    assert_eq!(ready.status(), StatusCode::OK);
    assert_eq!(ready.json()["status"], "READY");
    assert!(ready.json()["fulfilled"].as_bool().unwrap());

    let queried_preparation = send(
        &fixture.app,
        Method::GET,
        &format!(
            "/work-preparations/{}",
            fixture.assignment.work_assignment_id
        ),
        &admin,
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(queried_preparation.status(), StatusCode::OK);
    assert_eq!(queried_preparation.json()["status"], "READY");
}
