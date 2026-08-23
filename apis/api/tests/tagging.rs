//! v2 opaque NFC token tagging contract tests.

mod common;

use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;
use vespera::axum::http::StatusCode;

use api::models::{
    equipment_profiles::{self, EquipmentLifecycleStatus, EquipmentOwnershipType},
    equipment_tag_tokens, equipment_types, v2_work_assignments,
    works::{self, WorkLifecycleStatus},
};

fn key() -> String {
    Uuid::new_v4().to_string()
}

async fn fixture(
    app: &common::TestApp,
    owner_employee_id: Option<i64>,
    ownership: EquipmentOwnershipType,
    status: EquipmentLifecycleStatus,
) -> (api::models::employees::Model, i64, String) {
    let (worker, _) = common::spawn_worker(&app.db, "v2-tag-worker").await;
    let owner = owner_employee_id.or(Some(worker.employee_id));
    let profile = common::seed_v2_equipment(&app.db, owner, ownership, status).await;
    let assignment =
        common::seed_v2_assignment(&app.db, worker.employee_id, profile.equipment_type_id).await;
    let (_, token) =
        common::seed_v2_token(&app.db, profile.equipment_profile_id, worker.employee_id).await;
    (worker, assignment.work_assignment_id, token)
}

#[tokio::test]
async fn personal_owner_is_accepted_and_non_owner_is_rejected() {
    let app = common::spawn_app().await;
    let (owner, assignment_id, token) = fixture(
        &app,
        None,
        EquipmentOwnershipType::Personal,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let first = app
        .tag_equipment_with(
            &common::bearer_for(owner.employee_id, "WORKER"),
            assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(first.status(), StatusCode::CREATED);
    assert_eq!(first.json()["reason_code"], "TAG_ACCEPTED");

    let (other, _) = common::spawn_worker(&app.db, "v2-tag-other").await;
    let other_assignment = common::seed_v2_assignment(&app.db, other.employee_id, 1).await;
    let response = app
        .tag_equipment_with(
            &common::bearer_for(other.employee_id, "WORKER"),
            other_assignment.work_assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.json()["reason_code"],
        "PERSONAL_EQUIPMENT_OWNER_MISMATCH"
    );
}

#[tokio::test]
async fn personal_profile_rejects_a_different_worker_with_a_known_token() {
    let app = common::spawn_app().await;
    let (owner, _assignment_id, token) = fixture(
        &app,
        None,
        EquipmentOwnershipType::Personal,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let (other, _) = common::spawn_worker(&app.db, "v2-personal-non-owner").await;
    let profile = equipment_profiles::Entity::find()
        .one(&app.db)
        .await
        .expect("profile query")
        .expect("profile");
    let assignment =
        common::seed_v2_assignment(&app.db, other.employee_id, profile.equipment_type_id).await;
    // The worker's assignment is valid, but the token is resolved to a personal profile owned by owner.
    let response = app
        .tag_equipment_with(
            &common::bearer_for(other.employee_id, "WORKER"),
            assignment.work_assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.json()["reason_code"],
        "PERSONAL_EQUIPMENT_OWNER_MISMATCH"
    );
    assert_ne!(owner.employee_id, other.employee_id);
}

#[tokio::test]
async fn shared_equipment_conflicts_across_active_assignments() {
    let app = common::spawn_app().await;
    let (first, first_assignment, token) = fixture(
        &app,
        None,
        EquipmentOwnershipType::Shared,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let first_response = app
        .tag_equipment_with(
            &common::bearer_for(first.employee_id, "WORKER"),
            first_assignment,
            &token,
            &key(),
        )
        .await;
    assert_eq!(first_response.status(), StatusCode::CREATED);

    let (second, _) = common::spawn_worker(&app.db, "v2-shared-second").await;
    let profile = equipment_profiles::Entity::find()
        .one(&app.db)
        .await
        .expect("profile query")
        .expect("profile");
    let second_assignment =
        common::seed_v2_assignment(&app.db, second.employee_id, profile.equipment_type_id).await;
    let response = app
        .tag_equipment_with(
            &common::bearer_for(second.employee_id, "WORKER"),
            second_assignment.work_assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(response.json()["reason_code"], "SHARED_EQUIPMENT_IN_USE");
}

#[tokio::test]
async fn wrong_equipment_type_is_rejected_with_a_structured_reason() {
    let app = common::spawn_app().await;
    let (worker, _) = common::spawn_worker(&app.db, "v2-wrong-type").await;
    let matched = common::seed_v2_equipment(
        &app.db,
        None,
        EquipmentOwnershipType::Shared,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let assignment =
        common::seed_v2_assignment(&app.db, worker.employee_id, matched.equipment_type_id).await;
    let different_type = equipment_types::ActiveModel {
        equipment_type_id: sea_orm::ActiveValue::NotSet,
        name: Set("다른 장비 종류".to_string()),
        category: Set(Some("OTHER".to_string())),
        description: Set(None),
        created_at: sea_orm::ActiveValue::NotSet,
        updated_at: Set(None),
    }
    .insert(&app.db)
    .await
    .expect("다른 장비 종류 삽입")
    .equipment_type_id;
    let profile = equipment_profiles::ActiveModel {
        equipment_profile_id: sea_orm::ActiveValue::NotSet,
        legacy_equipment_id: Set(None),
        equipment_type_id: Set(different_type),
        asset_number: Set(Some("V2-WRONG-TYPE".to_string())),
        ownership_type: Set(EquipmentOwnershipType::Shared),
        owner_employee_id: Set(None),
        status: Set(EquipmentLifecycleStatus::Available),
        status_reason: Set(None),
        braille_label: Set(None),
        manufacturer_replacement_due_at: Set(None),
        created_at: sea_orm::ActiveValue::NotSet,
        updated_at: Set(None),
    }
    .insert(&app.db)
    .await
    .expect("다른 장비 프로필 삽입");
    let (_, token) =
        common::seed_v2_token(&app.db, profile.equipment_profile_id, worker.employee_id).await;
    let response = app
        .tag_equipment_with(
            &common::bearer_for(worker.employee_id, "WORKER"),
            assignment.work_assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(response.json()["reason_code"], "PPE_REQUIREMENT_MISMATCH");
}

#[tokio::test]
async fn all_non_available_lifecycle_states_are_rejected_with_stable_codes() {
    let cases = [
        (EquipmentLifecycleStatus::Blocked, "EQUIPMENT_BLOCKED"),
        (EquipmentLifecycleStatus::Damaged, "EQUIPMENT_DAMAGED"),
        (EquipmentLifecycleStatus::Lost, "EQUIPMENT_LOST"),
        (EquipmentLifecycleStatus::Replaced, "EQUIPMENT_REPLACED"),
    ];
    for (status, reason_code) in cases {
        let app = common::spawn_app().await;
        let (worker, assignment_id, token) =
            fixture(&app, None, EquipmentOwnershipType::Shared, status).await;
        let response = app
            .tag_equipment_with(
                &common::bearer_for(worker.employee_id, "WORKER"),
                assignment_id,
                &token,
                &key(),
            )
            .await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response.json()["reason_code"], reason_code);
    }
}

#[tokio::test]
async fn shared_claim_is_retained_while_stopped_and_released_after_completion() {
    let app = common::spawn_app().await;
    let (first, first_assignment_id, token) = fixture(
        &app,
        None,
        EquipmentOwnershipType::Shared,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let first_response = app
        .tag_equipment_with(
            &common::bearer_for(first.employee_id, "WORKER"),
            first_assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(first_response.status(), StatusCode::CREATED);

    let first_assignment = v2_work_assignments::Entity::find_by_id(first_assignment_id)
        .one(&app.db)
        .await
        .expect("assignment query")
        .expect("assignment");
    let first_work = works::Entity::find_by_id(first_assignment.work_id)
        .one(&app.db)
        .await
        .expect("work query")
        .expect("work");
    let profile = equipment_profiles::Entity::find()
        .one(&app.db)
        .await
        .expect("profile query")
        .expect("profile");
    let (second, _) = common::spawn_worker(&app.db, "v2-lifecycle-second").await;
    let second_assignment =
        common::seed_v2_assignment(&app.db, second.employee_id, profile.equipment_type_id).await;

    let mut stopped: works::ActiveModel = first_work.clone().into();
    stopped.status = Set(WorkLifecycleStatus::Stopped);
    stopped.update(&app.db).await.expect("stop work");
    let stopped_response = app
        .tag_equipment_with(
            &common::bearer_for(second.employee_id, "WORKER"),
            second_assignment.work_assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(stopped_response.status(), StatusCode::CONFLICT);
    assert_eq!(
        stopped_response.json()["reason_code"],
        "SHARED_EQUIPMENT_IN_USE"
    );

    let mut completed: works::ActiveModel = first_work.into();
    completed.status = Set(WorkLifecycleStatus::Completed);
    completed.update(&app.db).await.expect("complete work");
    let released_response = app
        .tag_equipment_with(
            &common::bearer_for(second.employee_id, "WORKER"),
            second_assignment.work_assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(released_response.status(), StatusCode::CREATED);
    assert_eq!(released_response.json()["reason_code"], "TAG_ACCEPTED");
}

#[tokio::test]
async fn unknown_and_deactivated_tokens_are_not_registered() {
    let app = common::spawn_app().await;
    let (worker, assignment_id, token) = fixture(
        &app,
        None,
        EquipmentOwnershipType::Shared,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let unknown = app
        .tag_equipment_with(
            &common::bearer_for(worker.employee_id, "WORKER"),
            assignment_id,
            "eqt_unknown",
            &key(),
        )
        .await;
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    assert_eq!(unknown.json()["reason_code"], "TAG_NOT_REGISTERED");

    let token_row = equipment_tag_tokens::Entity::find()
        .one(&app.db)
        .await
        .expect("token query")
        .expect("token");
    let mut revoked: equipment_tag_tokens::ActiveModel = token_row.into();
    revoked.is_active = Set(false);
    revoked.update(&app.db).await.expect("deactivate token");
    let deactivated = app
        .tag_equipment_with(
            &common::bearer_for(worker.employee_id, "WORKER"),
            assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(deactivated.status(), StatusCode::NOT_FOUND);
    assert_eq!(deactivated.json()["reason_code"], "TAG_NOT_REGISTERED");
}

#[tokio::test]
async fn same_key_replays_exact_response_and_changed_payload_is_conflict() {
    let app = common::spawn_app().await;
    let (worker, assignment_id, token) = fixture(
        &app,
        None,
        EquipmentOwnershipType::Shared,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let idempotency_key = key();
    let first = app
        .tag_equipment_with(
            &common::bearer_for(worker.employee_id, "WORKER"),
            assignment_id,
            &token,
            &idempotency_key,
        )
        .await;
    let first_body = first.json();
    assert_eq!(first.status(), StatusCode::CREATED);
    let replay = app
        .tag_equipment_with(
            &common::bearer_for(worker.employee_id, "WORKER"),
            assignment_id,
            &token,
            &idempotency_key,
        )
        .await;
    assert_eq!(replay.status(), StatusCode::CREATED);
    assert_eq!(replay.json(), first_body);

    let changed = app
        .tag_equipment_with(
            &common::bearer_for(worker.employee_id, "WORKER"),
            assignment_id,
            "eqt_changed",
            &idempotency_key,
        )
        .await;
    assert_eq!(changed.status(), StatusCode::CONFLICT);
    assert_eq!(changed.json()["reason_code"], "IDEMPOTENCY_KEY_REUSED");
}

#[tokio::test]
async fn new_key_duplicate_returns_already_accepted_without_a_second_allocation() {
    let app = common::spawn_app().await;
    let (worker, assignment_id, token) = fixture(
        &app,
        None,
        EquipmentOwnershipType::Shared,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let first = app
        .tag_equipment_with(
            &common::bearer_for(worker.employee_id, "WORKER"),
            assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(first.status(), StatusCode::CREATED);
    let duplicate = app
        .tag_equipment_with(
            &common::bearer_for(worker.employee_id, "WORKER"),
            assignment_id,
            &token,
            &key(),
        )
        .await;
    assert_eq!(duplicate.status(), StatusCode::OK);
    assert_eq!(duplicate.json()["reason_code"], "TAG_ALREADY_ACCEPTED");
    assert_eq!(
        api::models::work_assignment_equipment::Entity::find()
            .all(&app.db)
            .await
            .expect("allocation query")
            .len(),
        1
    );
}

#[tokio::test]
async fn admin_issue_and_revoke_only_stores_a_hash() {
    let app = common::spawn_app().await;
    let (_worker, _, _) = fixture(
        &app,
        None,
        EquipmentOwnershipType::Shared,
        EquipmentLifecycleStatus::Available,
    )
    .await;
    let profile = equipment_profiles::Entity::find()
        .one(&app.db)
        .await
        .expect("profile query")
        .expect("profile");
    // Remove the fixture token so the admin endpoint exercises issuance.
    equipment_tag_tokens::Entity::delete_many()
        .exec(&app.db)
        .await
        .expect("delete fixture token");
    let issued = app
        .issue_equipment_tag_token_with(&common::admin_bearer(), profile.equipment_profile_id)
        .await;
    assert_eq!(issued.status(), StatusCode::CREATED);
    let body = issued.json();
    let raw_token = body["token"].as_str().expect("issued token").to_string();
    let row = equipment_tag_tokens::Entity::find()
        .one(&app.db)
        .await
        .expect("token query")
        .expect("issued token row");
    assert_ne!(row.tag_token_hash, raw_token);
    assert_eq!(row.tag_token_hash.len(), 64);
    let revoked = app
        .revoke_equipment_tag_token_with(
            &common::admin_bearer(),
            body["equipment_tag_token_id"].as_i64().unwrap(),
        )
        .await;
    assert_eq!(revoked.status(), StatusCode::OK);
}
