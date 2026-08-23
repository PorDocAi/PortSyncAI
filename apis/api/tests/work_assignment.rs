//! 작업 배정 시나리오 통합 테스트 (AC-8 — #42)
//!
//! - AC-8: MSDS 검수 미확정(PENDING) 문서에 연결된 화물로 작업 배정 생성 → 422와
//!   구조화된 사유 코드(MSDS_REVIEW_NOT_CONFIRMED) 반환, 배정 행은 생기지 않는다.
//! - 대조군: 검수 확정(CONFIRMED) 문서의 화물에는 배정이 생성된다.

mod common;

use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set};

use api::models::{
    cargo_document_versions::{
        self, CargoProcessingStatus, CargoReviewStatus, V2CargoDocumentType,
    },
    cargo_documents::{self, CargoDocumentType, FileFormat, ReviewStatus},
    cargo_item_documents::{self, CargoDocumentRole},
    cargo_items, work_assignments,
};
use common::TestApp;
use vespera::axum::http::StatusCode;

/// AC-8 — 검수 미확정(PENDING) 화물에 배정하면 422 + 사유 코드
#[tokio::test]
async fn assignment_to_unconfirmed_msds_cargo_is_rejected_with_422() {
    let app = spawn_assignment_app().await;
    let admin_bearer = common::bearer_for(1, "ADMIN");
    let (worker, _) = common::spawn_worker(&app.db, "nfc-work-assignment").await;
    let item = seed_cargo_item(&app, ReviewStatus::Pending).await;

    let response = app
        .create_assignment_with(&admin_bearer, worker.employee_id, Some(item.cargo_item_id))
        .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = response.json();
    assert_eq!(body["code"], "MSDS_REVIEW_NOT_CONFIRMED");
    assert!(
        body["message"].as_str().unwrap().contains("MSDS"),
        "한국어 사유에 MSDS 안내가 포함돼야 한다: {}",
        body["message"]
    );

    let assignments = assignments_for(&app, worker.employee_id).await;
    assert!(
        assignments.is_empty(),
        "차단된 배정은 행으로 남지 않아야 한다"
    );
}

/// 매핑된 v2 MSDS가 미확정이면 레거시 문서가 확정이어도 배정을 차단한다
#[tokio::test]
async fn assignment_to_mapped_unconfirmed_v2_msds_is_rejected() {
    let app = spawn_assignment_app().await;
    let admin_bearer = common::bearer_for(1, "ADMIN");
    let (worker, _) = common::spawn_worker(&app.db, "nfc-v2-work-assignment").await;
    let item = seed_cargo_item(&app, ReviewStatus::Confirmed).await;
    seed_v2_msds(&app, item.cargo_item_id, CargoReviewStatus::Pending).await;

    let response = app
        .create_assignment_with(&admin_bearer, worker.employee_id, Some(item.cargo_item_id))
        .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response.json()["code"], "MSDS_REVIEW_NOT_CONFIRMED");
}

/// 매핑된 v2 MSDS가 확정이면 배정을 생성한다
#[tokio::test]
async fn assignment_to_mapped_confirmed_v2_msds_is_created() {
    let app = spawn_assignment_app().await;
    let admin_bearer = common::bearer_for(1, "ADMIN");
    let (worker, _) = common::spawn_worker(&app.db, "nfc-v2-work-assignment").await;
    let item = seed_cargo_item(&app, ReviewStatus::Confirmed).await;
    seed_v2_msds(&app, item.cargo_item_id, CargoReviewStatus::Confirmed).await;

    let response = app
        .create_assignment_with(&admin_bearer, worker.employee_id, Some(item.cargo_item_id))
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
}

/// 검수 확정(CONFIRMED) 화물에는 배정이 생성된다 (차단 규칙의 대조군)
#[tokio::test]
async fn assignment_to_confirmed_cargo_is_created() {
    let app = spawn_assignment_app().await;
    let admin_bearer = common::bearer_for(1, "ADMIN");
    let (worker, _) = common::spawn_worker(&app.db, "nfc-work-assignment").await;
    let item = seed_cargo_item(&app, ReviewStatus::Confirmed).await;

    let response = app
        .create_assignment_with(&admin_bearer, worker.employee_id, Some(item.cargo_item_id))
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.json();
    assert_eq!(body["cargo_item_id"], item.cargo_item_id);
    assert_eq!(body["eligibility_status"], "ELIGIBLE");

    let assignments = assignments_for(&app, worker.employee_id).await;
    assert_eq!(assignments.len(), 1);
}

/// 화물을 지정하지 않은 공용 작업 배정은 검수 검증 없이 생성된다
#[tokio::test]
async fn assignment_without_cargo_item_is_created() {
    let app = spawn_assignment_app().await;
    let admin_bearer = common::bearer_for(1, "ADMIN");
    let (worker, _) = common::spawn_worker(&app.db, "nfc-work-assignment").await;

    let response = app
        .create_assignment_with(&admin_bearer, worker.employee_id, None)
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
}

/// 존재하지 않는 화물 지정은 404와 사유 코드로 거부된다
#[tokio::test]
async fn assignment_to_missing_cargo_item_is_not_found() {
    let app = spawn_assignment_app().await;
    let admin_bearer = common::bearer_for(1, "ADMIN");
    let (worker, _) = common::spawn_worker(&app.db, "nfc-work-assignment").await;

    let response = app
        .create_assignment_with(&admin_bearer, worker.employee_id, Some(999_999))
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.json()["code"], "CARGO_ITEM_NOT_FOUND");
}

async fn spawn_assignment_app() -> TestApp {
    common::spawn_app_with(|router| router).await
}

/// 출처 문서(검수 상태 포함) + 그 문서에 연결된 화물 픽스처
async fn seed_cargo_item(app: &TestApp, review_status: ReviewStatus) -> cargo_items::Model {
    let (uploader, _) = common::spawn_worker(&app.db, "nfc-doc-uploader").await;

    let document = cargo_documents::ActiveModel {
        cargo_document_id: NotSet,
        document_type: Set(CargoDocumentType::Bl),
        file_format: Set(FileFormat::Pdf),
        file_url: Set(format!("uploads/{}.pdf", uuid::Uuid::new_v4())),
        original_file_name: Set(Some("BL.pdf".to_string())),
        content_type: Set(Some("application/pdf".to_string())),
        file_size: Set(Some(1024)),
        file_hash: Set(None),
        review_status: Set(review_status),
        uploaded_by_id: Set(uploader.employee_id),
        created_at: NotSet,
    }
    .insert(&app.db)
    .await
    .expect("화물문서 픽스처 삽입 실패");

    cargo_items::ActiveModel {
        cargo_item_id: NotSet,
        cargo_document_id: Set(Some(document.cargo_document_id)),
        bl_number: Set(Some(format!("BL-{}", uuid::Uuid::new_v4().simple()))),
        dgd_number: Set(None),
        un_number: Set(None),
        dg_class_id: Set(None),
        hs_code: Set(None),
        item_name: Set(Some("테스트 화물".to_string())),
        description: Set(None),
        is_dangerous: Set(false),
        dgd_missing_warning: Set(false),
        arrival_date: Set(Some(api::routes::attendances::today())),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(&app.db)
    .await
    .expect("화물 픽스처 삽입 실패")
}

async fn seed_v2_msds(app: &TestApp, cargo_item_id: i64, review_status: CargoReviewStatus) {
    let version = cargo_document_versions::ActiveModel {
        cargo_document_version_id: sea_orm::ActiveValue::NotSet,
        legacy_cargo_document_id: sea_orm::ActiveValue::NotSet,
        document_type: Set(V2CargoDocumentType::Msds),
        document_version: Set(1),
        processing_status: Set(CargoProcessingStatus::Completed),
        review_status: Set(review_status),
        reviewed_by_id: sea_orm::ActiveValue::NotSet,
        reviewed_at: sea_orm::ActiveValue::NotSet,
        created_at: sea_orm::ActiveValue::NotSet,
        updated_at: sea_orm::ActiveValue::NotSet,
    }
    .insert(&app.db)
    .await
    .expect("v2 문서 삽입 실패");
    cargo_item_documents::ActiveModel {
        cargo_item_document_id: sea_orm::ActiveValue::NotSet,
        cargo_item_id: Set(cargo_item_id),
        cargo_document_version_id: Set(version.cargo_document_version_id),
        document_role: Set(CargoDocumentRole::Msds),
        is_primary: Set(true),
        created_at: sea_orm::ActiveValue::NotSet,
    }
    .insert(&app.db)
    .await
    .expect("v2 문서 매핑 삽입 실패");
}

async fn assignments_for(app: &TestApp, employee_id: i64) -> Vec<work_assignments::Model> {
    work_assignments::Entity::find()
        .filter(work_assignments::Column::EmployeeId.eq(employee_id))
        .all(&app.db)
        .await
        .expect("작업 배정 조회 실패")
}
