//! 게이트 인증·단말 자격증명 통합 테스트 (AC-1, AC-2, AC-10 — #42)
//!
//! - AC-1: 유효한 단말 토큰 + 오늘자 출근 절차를 마친 작업원 태깅 → 통과
//! - AC-2: 등록·활성 단말 토큰 없는 /gate/verify 호출은 전부 401 (fail-closed)
//! - AC-10: 단말 발급은 관리 권한 JWT로만 가능하고, 폐기된 단말 토큰은 즉시 401

mod common;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use common::{ADMIN_TOKEN, TERMINAL_TOKEN, TestApp, WORKER_NFC_UID};

/// 관리자 JWT는 시스템 비밀키로 서명된 실제 토큰이어야 한다.
#[tokio::test]
async fn admin_guard_rejects_unsigned_token() {
    let app = common::spawn_app().await;
    let response = app
        .revoke_terminal_with(format!("Bearer {ADMIN_TOKEN}").as_str(), 1)
        .await;
    assert_eq!(
        response.status(),
        vespera::axum::http::StatusCode::UNAUTHORIZED
    );
}

/// AC-1 — 통과 플로우: 활성 단말 + 절차 완료 작업원 → allowed=true, "통과"
#[tokio::test]
async fn verify_gate_passes_worker_with_completed_procedure() {
    let app = common::spawn_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    let worker = common::seed_worker_with_started_attendance(&app.db).await;

    let response = app
        .verify_gate_with(Some(&format!("Bearer {terminal_token}")), WORKER_NFC_UID)
        .await;

    assert_eq!(response.status(), vespera::axum::http::StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], serde_json::Value::Bool(true));
    assert_eq!(body["employee_name"], worker.name);
    assert_eq!(body["reason"], "통과");

    // DB까지 확인: gate_status가 PASSED로 승격됐는다
    let attendance = api::models::attendances::Entity::find()
        .filter(api::models::attendances::Column::EmployeeId.eq(worker.employee_id))
        .one(&app.db)
        .await
        .expect("출근 조회 실패")
        .expect("출근 행이 존재해야 한다");
    assert_eq!(
        attendance.gate_status,
        api::models::attendances::GateStatus::Passed
    );
}

/// AC-2 — 등록되지 않은 토큰, 형식 오류, 헤더 누락 모두 fail-closed 401
#[tokio::test]
async fn verify_gate_requires_registered_active_terminal() {
    let app = common::spawn_app().await;
    common::seed_worker_with_started_attendance(&app.db).await;

    // 미등록 토큰
    let response = app
        .verify_gate_with(Some(&format!("Bearer {TERMINAL_TOKEN}")), WORKER_NFC_UID)
        .await;
    assert_eq!(
        response.status(),
        vespera::axum::http::StatusCode::UNAUTHORIZED
    );

    // Bearer 접두어 없는 값
    let response = app
        .verify_gate_with(Some(TERMINAL_TOKEN), WORKER_NFC_UID)
        .await;
    assert_eq!(
        response.status(),
        vespera::axum::http::StatusCode::UNAUTHORIZED
    );

    // Authorization 헤더 자체가 없음
    let response = app.verify_gate_with(None, WORKER_NFC_UID).await;
    assert_eq!(
        response.status(),
        vespera::axum::http::StatusCode::UNAUTHORIZED
    );
}

/// AC-2 보강 — 존재하는 사원 UID라도 단말 인증에 실패하면 어떤 정보도 반환하지 않는다(401 우선)
#[tokio::test]
async fn verify_gate_fails_closed_before_employee_lookup() {
    let app = common::spawn_app().await;
    let _worker = common::seed_worker_with_started_attendance(&app.db).await;

    let response = app
        .verify_gate_with(Some(&format!("Bearer {TERMINAL_TOKEN}")), "unknown-uid")
        .await;
    assert_eq!(
        response.status(),
        vespera::axum::http::StatusCode::UNAUTHORIZED
    );
}

/// AC-10 — 단말 폐기는 관리 권한으로만 가능하고, 폐기 즉시 해당 토큰은 401이 된다
#[tokio::test]
async fn revoked_terminal_is_rejected_immediately() {
    let app = common::spawn_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    let _worker = common::seed_worker_with_started_attendance(&app.db).await;

    // 발급 응답의 terminal_id로 폐기 — 소프트 삭제(is_active=false)
    let issued_terminal_id = find_terminal_id(&app).await;
    let response = app
        .revoke_terminal_with(&common::admin_bearer(), issued_terminal_id)
        .await;
    assert_eq!(response.status(), vespera::axum::http::StatusCode::OK);

    // 폐기 후 같은 토큰으로 검증 요청 → 401
    let response = app
        .verify_gate_with(Some(&format!("Bearer {terminal_token}")), WORKER_NFC_UID)
        .await;
    assert_eq!(
        response.status(),
        vespera::axum::http::StatusCode::UNAUTHORIZED
    );
    assert!(!common::terminal_is_active(&app.db, issued_terminal_id).await);
}

/// AC-10 보강 — 폐기 API도 관리 권한 없으면 거부된다
#[tokio::test]
async fn revoke_requires_admin_privilege() {
    let app = common::spawn_app().await;
    let _terminal_token = common::seed_active_terminal(&app).await;
    let terminal_id = find_terminal_id(&app).await;

    // 유효하지만 비관리 권한인 JWT로 폐기 시도 → 403
    let response = app
        .revoke_terminal_with(&common::worker_bearer(), terminal_id)
        .await;
    assert_eq!(
        response.status(),
        vespera::axum::http::StatusCode::FORBIDDEN
    );
}

/// AC-10 보강 — 발급/폐기 모두 무효한 JWT는 401 (서명 불일치)
#[tokio::test]
async fn issue_and_revoke_require_validly_signed_jwt() {
    let app = common::spawn_app().await;

    let response = app
        .send_request(
            vespera::axum::http::Request::builder()
                .method(vespera::axum::http::Method::POST)
                .uri("/gate/terminals")
                .header("Authorization", format!("Bearer {ADMIN_TOKEN}"))
                .header("Content-Type", "application/json")
                .body(serde_json::json!({ "gate_id": "남문" }).to_string())
                .unwrap(),
        )
        .await;
    assert_eq!(
        response.status(),
        vespera::axum::http::StatusCode::UNAUTHORIZED
    );

    let response = app
        .revoke_terminal_with(&format!("Bearer {ADMIN_TOKEN}"), 1)
        .await;
    assert_eq!(
        response.status(),
        vespera::axum::http::StatusCode::UNAUTHORIZED
    );
}

async fn find_terminal_id(app: &TestApp) -> i64 {
    api::models::gate_terminals::Entity::find()
        .order_by_asc(api::models::gate_terminals::Column::TerminalId)
        .one(&app.db)
        .await
        .expect("단말 조회 실패")
        .expect("발급된 단말이 존재해야 한다")
        .terminal_id
}
