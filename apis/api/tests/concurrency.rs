//! v2 NFC tagging concurrency tests.
//!
//! Every request is a real in-process Axum future. No sleeps or polling are used.

mod common;

use std::sync::Arc;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::task::JoinSet;
use uuid::Uuid;
use vespera::axum::http::StatusCode;

use api::models::{
    equipment_check_events, equipment_profiles::EquipmentLifecycleStatus,
    equipment_profiles::EquipmentOwnershipType, equipment_tag_tokens, shared_equipment_claims,
    work_assignment_equipment,
};

fn key() -> String {
    Uuid::new_v4().to_string()
}

/// Distinct active assignments racing for one shared asset produce one accepted
/// claim and nine portable conflict responses.
#[tokio::test]
async fn ten_concurrent_shared_tags_yield_one_acceptance_and_nine_conflicts() {
    let app = Arc::new(common::spawn_app().await);
    let (issuer, _) = common::spawn_worker(&app.db, "v2-concurrent-issuer").await;
    let profile = common::seed_v2_equipment(
        &app.db,
        None,
        EquipmentOwnershipType::Shared,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let (_, token) =
        common::seed_v2_token(&app.db, profile.equipment_profile_id, issuer.employee_id).await;

    let mut participants = Vec::with_capacity(10);
    for index in 0..10 {
        let (worker, _) = common::spawn_worker(&app.db, &format!("v2-concurrent-{index:02}")).await;
        let assignment =
            common::seed_v2_assignment(&app.db, worker.employee_id, profile.equipment_type_id)
                .await;
        participants.push((
            worker.employee_id,
            assignment.work_assignment_id,
            common::bearer_for(worker.employee_id, "WORKER"),
        ));
    }

    let mut set = JoinSet::new();
    for (employee_id, assignment_id, bearer) in participants {
        let app = Arc::clone(&app);
        let token = token.clone();
        set.spawn(async move {
            let response = app
                .tag_equipment_with(&bearer, assignment_id, &token, &key())
                .await;
            (employee_id, response)
        });
    }

    let mut results = Vec::with_capacity(10);
    while let Some(joined) = set.join_next().await {
        results.push(joined.expect("동시 태깅 태스크 패닉"));
    }

    assert_eq!(results.len(), 10);
    assert_eq!(
        results
            .iter()
            .filter(|(_, response)| response.status() == StatusCode::CREATED)
            .count(),
        1,
        "공용 장비에 대한 첫 수락은 정확히 하나여야 한다"
    );
    assert_eq!(
        results
            .iter()
            .filter(|(_, response)| {
                response.status() == StatusCode::CONFLICT
                    && response.json()["reason_code"] == "SHARED_EQUIPMENT_IN_USE"
            })
            .count(),
        9,
        "나머지는 모두 SHARED_EQUIPMENT_IN_USE여야 한다"
    );
    assert!(results.iter().all(|(_, response)| {
        response.status() == StatusCode::CREATED
            || (response.status() == StatusCode::CONFLICT
                && response.json()["reason_code"] == "SHARED_EQUIPMENT_IN_USE")
    }));

    let claims = shared_equipment_claims::Entity::find()
        .all(&app.db)
        .await
        .expect("공용 점유 조회 실패");
    assert_eq!(claims.len(), 1);
    let allocations = work_assignment_equipment::Entity::find()
        .filter(
            work_assignment_equipment::Column::EquipmentProfileId.eq(profile.equipment_profile_id),
        )
        .all(&app.db)
        .await
        .expect("장비 할당 조회 실패");
    assert_eq!(allocations.len(), 1);
    let events = equipment_check_events::Entity::find()
        .all(&app.db)
        .await
        .expect("태깅 이벤트 조회 실패");
    assert_eq!(events.len(), 10);
}

/// Concurrent copies of one request use the worker/key uniqueness boundary and
/// return the same stored response rather than creating duplicate allocations.
#[tokio::test]
async fn concurrent_same_key_requests_replay_one_stored_response() {
    let app = Arc::new(common::spawn_app().await);
    let (worker, _) = common::spawn_worker(&app.db, "v2-concurrent-same-key").await;
    let profile = common::seed_v2_equipment(
        &app.db,
        None,
        EquipmentOwnershipType::Shared,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let (_, token) =
        common::seed_v2_token(&app.db, profile.equipment_profile_id, worker.employee_id).await;
    let assignment =
        common::seed_v2_assignment(&app.db, worker.employee_id, profile.equipment_type_id).await;
    let bearer = common::bearer_for(worker.employee_id, "WORKER");
    let idempotency_key = key();

    let mut set = JoinSet::new();
    for _ in 0..10 {
        let app = Arc::clone(&app);
        let bearer = bearer.clone();
        let token = token.clone();
        let idempotency_key = idempotency_key.clone();
        let assignment_id = assignment.work_assignment_id;
        set.spawn(async move {
            app.tag_equipment_with(&bearer, assignment_id, &token, &idempotency_key)
                .await
        });
    }

    let mut responses = Vec::with_capacity(10);
    while let Some(joined) = set.join_next().await {
        responses.push(joined.expect("동일 멱등키 태스크 패닉"));
    }
    assert_eq!(responses.len(), 10);
    assert!(responses.iter().all(|response| {
        response.status() == StatusCode::CREATED && response.json()["reason_code"] == "TAG_ACCEPTED"
    }));
    let allocations = work_assignment_equipment::Entity::find()
        .all(&app.db)
        .await
        .expect("장비 할당 조회 실패");
    assert_eq!(allocations.len(), 1);
    let events = equipment_check_events::Entity::find()
        .all(&app.db)
        .await
        .expect("태깅 이벤트 조회 실패");
    assert_eq!(events.len(), 1);
    let tokens = equipment_tag_tokens::Entity::find()
        .all(&app.db)
        .await
        .expect("토큰 조회 실패");
    assert_eq!(tokens.len(), 1);
}
