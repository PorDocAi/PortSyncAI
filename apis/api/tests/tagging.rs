//! 장비 NFC 태깅 시나리오 통합 테스트 (AC-6 — #42)
//!
//! - AC-6: 같은 장비를 오늘 두 번째 태깅하면 409로 거부되고, equipment_check_logs에는
//!   첫 태깅 1건만 유지된다 (돌려쓰기 차단 — 본인 재태깅·타인 사용 모두 포함).
//! - 비활성 장비 태깅은 422로 거부된다 (FR-D4 방어 경계).

mod common;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use api::models::equipment;
use common::TestApp;

fn conflict() -> vespera::axum::http::StatusCode {
    vespera::axum::http::StatusCode::CONFLICT
}

/// AC-6 — 동일 작업원이 같은 장비를 다시 태깅하면 409, 로그는 1건 유지
#[tokio::test]
async fn duplicate_equipment_tag_returns_conflict_and_keeps_single_log() {
    let app = common::spawn_app().await;
    let (worker, _attendance) = common::spawn_worker(&app.db, "nfc-tagging-self-dup").await;
    let device = common::seed_active_equipment(&app.db).await;
    let bearer = common::bearer_for(worker.employee_id, "WORKER");

    let first = app.tag_equipment_with(&bearer, &device.nfc_tag_uid).await;
    assert_eq!(first.status(), vespera::axum::http::StatusCode::CREATED);
    let body = first.json();
    assert_eq!(body["equipment_id"], device.equipment_id);
    assert_eq!(body["tagged_count_today"], 1);

    let second = app.tag_equipment_with(&bearer, &device.nfc_tag_uid).await;
    assert_eq!(second.status(), conflict());

    let logs = tagged_logs(&app, device.equipment_id).await;
    assert_eq!(
        logs.len(),
        1,
        "재태깅이 거부됐으면 로그는 첫 태깅 1건뿐이다"
    );
    assert_eq!(logs[0].employee_id, worker.employee_id);
}

/// AC-6 — 이미 사용 중인 장비를 다른 작업원이 태깅해도 409 (돌려쓰기 차단)
#[tokio::test]
async fn equipment_used_by_another_worker_returns_conflict() {
    let app = common::spawn_app().await;
    let (first_worker, _) = common::spawn_worker(&app.db, "nfc-tagging-first").await;
    let (second_worker, _) = common::spawn_worker(&app.db, "nfc-tagging-second").await;
    let device = common::seed_active_equipment(&app.db).await;

    let first = app
        .tag_equipment_with(
            &common::bearer_for(first_worker.employee_id, "WORKER"),
            &device.nfc_tag_uid,
        )
        .await;
    assert_eq!(first.status(), vespera::axum::http::StatusCode::CREATED);

    let second = app
        .tag_equipment_with(
            &common::bearer_for(second_worker.employee_id, "WORKER"),
            &device.nfc_tag_uid,
        )
        .await;
    assert_eq!(second.status(), conflict());

    let logs = tagged_logs(&app, device.equipment_id).await;
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].employee_id, first_worker.employee_id);
}

/// 비활성 장비 태깅은 422로 거부되고 로그가 남지 않는다
#[tokio::test]
async fn inactive_equipment_tag_is_unprocessable() {
    let app = common::spawn_app().await;
    let (worker, _) = common::spawn_worker(&app.db, "nfc-tagging-inactive").await;
    let device = common::seed_active_equipment(&app.db).await;

    let mut retired: equipment::ActiveModel = device.clone().into();
    retired.is_active = Set(false);
    retired.update(&app.db).await.expect("장비 비활성화 실패");

    let response = app
        .tag_equipment_with(
            &common::bearer_for(worker.employee_id, "WORKER"),
            &device.nfc_tag_uid,
        )
        .await;
    assert_eq!(
        response.status(),
        vespera::axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );
    assert!(tagged_logs(&app, device.equipment_id).await.is_empty());
}

async fn tagged_logs(
    app: &TestApp,
    equipment_id: i64,
) -> Vec<api::models::equipment_check_logs::Model> {
    api::models::equipment_check_logs::Entity::find()
        .filter(api::models::equipment_check_logs::Column::EquipmentId.eq(equipment_id))
        .all(&app.db)
        .await
        .expect("태깅 로그 조회 실패")
}
