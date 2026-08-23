//! 게이트 출입 사원증 태깅 시나리오 통합 테스트 (AC-3·4·5·9 — #42)
//!
//! - AC-3: 등록/미등록/퇴사자 카드 태깅 결과 (allowed와 한국어 사유)
//! - AC-4: 지침 미확인·장비 미완료·승인 대기 상태는 전부 차단되고 사유에 항목이 포함된다
//! - AC-5: 통과 처리 후 재태깅은 멱등 동작하고 통과 이벤트는 1건만 유지된다
//! - AC-9: 작업중지(STOPPED) 상태 태깅은 승인 예외로도 면제되지 않는다

mod common;

use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use api::models::{
    attendances::{self, ApprovalStatus, GateStatus, WorkStatus},
    employees,
};
use common::TestApp;
use vespera::axum::http::StatusCode;

/// 등록된 카드가 절차를 마쳤을 때 통과한다 (차단 판정들의 대조군)
#[tokio::test]
async fn registered_card_with_completed_procedure_passes() {
    let app = spawn_gate_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    let worker = common::seed_worker_with_started_attendance(&app.db).await;

    let response = gate_verify(&app, &terminal_token, common::WORKER_NFC_UID).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], true);
    assert_eq!(body["reason"], "통과");
    assert_eq!(body["employee_name"], worker.name);
}

/// AC-3 — 등록됐지만 오늘 출근 절차가 없는 카드는 차단된다
#[tokio::test]
async fn registered_card_without_attendance_is_blocked() {
    let app = spawn_gate_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    common::spawn_worker(&app.db, common::WORKER_NFC_UID).await;

    // 오늘자 출근 행을 지워 '절차 미시작' 상태로 만든다
    let attendance = today_attendance(&app)
        .await
        .expect("출근 행이 존재해야 한다");
    attendances::ActiveModel {
        attendance_id: Set(attendance.attendance_id),
        ..Default::default()
    }
    .delete(&app.db)
    .await
    .expect("출근 행 삭제 실패");

    let response = gate_verify(&app, &terminal_token, common::WORKER_NFC_UID).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], false);
    assert_eq!(body["reason"], "오늘 출근 절차가 시작되지 않았습니다.");
}

/// AC-3 — 미등록 카드는 404로 차단된다 (게이트 화면에는 직원 정보가 노출되지 않음)
#[tokio::test]
async fn unknown_card_is_rejected_with_not_found() {
    let app = spawn_gate_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    let _worker = common::spawn_worker(&app.db, common::WORKER_NFC_UID).await;

    let response = gate_verify(&app, &terminal_token, "nfc-unknown-card").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// AC-3 — 퇴사자 카드는 허용되지 않는다.
/// 퇴사자는 오늘자 출근 절차를 가질 수 없으므로(현행 도메인 규칙)
/// '절차 미시작' 차단 경로로 거부돼야 한다.
#[tokio::test]
async fn resigned_card_is_not_allowed() {
    let app = spawn_gate_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    let (resigned, _) = common::spawn_worker(&app.db, "nfc-resigned-card").await;

    let mut retired: employees::ActiveModel = resigned.into();
    retired.status = Set(api::models::employees::EmployeeStatus::Resigned);
    retired.resigned_at = Set(Some(chrono::Utc::now().into()));
    retired.update(&app.db).await.expect("퇴사 처리 실패");

    // 퇴사 처리 시 오늘자 출근 절차도 함께 무효화된다 (미시작과 동일 상태)
    let attendance = today_attendance(&app)
        .await
        .expect("출근 행이 존재해야 한다");
    attendances::ActiveModel {
        attendance_id: Set(attendance.attendance_id),
        ..Default::default()
    }
    .delete(&app.db)
    .await
    .expect("출근 행 삭제 실패");

    let response = gate_verify(&app, &terminal_token, "nfc-resigned-card").await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], false);
}

/// AC-5 — 통과 후 재태깅은 멱등: "이미 통과" 응답, 통과 이벤트 로그는 1건 유지
#[tokio::test]
async fn rescan_after_pass_is_idempotent_and_keeps_single_pass_event() {
    let app = spawn_gate_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    common::seed_worker_with_started_attendance(&app.db).await;

    let first = gate_verify(&app, &terminal_token, common::WORKER_NFC_UID).await;
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(first.json()["reason"], "통과");

    let second = gate_verify(&app, &terminal_token, common::WORKER_NFC_UID).await;
    assert_eq!(second.status(), StatusCode::OK);
    let body = second.json();
    assert_eq!(body["allowed"], true);
    assert_eq!(body["reason"], "이미 통과 처리된 출근입니다.");

    // 재태깅으로 새 이벤트가 기록되지 않는다 — 게이트 로그 자체가 통과 1건뿐
    let logs = api::models::gate_verify_logs::Entity::find()
        .all(&app.db)
        .await
        .expect("게이트 검증 로그 조회 실패");
    assert_eq!(logs.len(), 1, "재태깅이 로그를 추가해선 안 된다");
    assert!(logs[0].is_pass_event);
    assert!(logs[0].allowed);

    // 상태는 여전히 PASSED로 유지된다
    let attendance = today_attendance(&app)
        .await
        .expect("출근 행이 존재해야 한다");
    assert_eq!(attendance.gate_status, GateStatus::Passed);
}

/// AC-4 — 안전지침 미확인은 차단 사유에 포함된다
#[tokio::test]
async fn unacknowledged_instruction_blocks_entry() {
    let app = spawn_gate_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    seed_attendance_with(
        &app,
        |a| {
            a.instruction_ack_completed = Set(false);
        },
        ApprovalStatus::NotRequired,
        WorkStatus::Normal,
    )
    .await;

    let response = gate_verify(&app, &terminal_token, common::WORKER_NFC_UID).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], false);
    assert!(
        body["reason"].as_str().unwrap().contains("안전지침"),
        "차단 사유에 지침 미확인이 포함돼야 한다: {}",
        body["reason"]
    );
}

/// AC-4 — 필수 장비 확인 미완료는 차단 사유에 포함된다
#[tokio::test]
async fn incomplete_equipment_check_blocks_entry() {
    let app = spawn_gate_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    seed_attendance_with(
        &app,
        |a| {
            a.equipment_check_completed = Set(false);
        },
        ApprovalStatus::NotRequired,
        WorkStatus::Normal,
    )
    .await;

    let response = gate_verify(&app, &terminal_token, common::WORKER_NFC_UID).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], false);
    assert!(
        body["reason"]
            .as_str()
            .unwrap()
            .contains("장비 확인 미완료"),
        "차단 사유에 장비 미완료가 포함돼야 한다: {}",
        body["reason"]
    );
}

/// AC-4 — 관리자 승인 대기 상태는 차단 사유에 포함된다
#[tokio::test]
async fn pending_approval_blocks_entry() {
    let app = spawn_gate_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    seed_attendance_with(&app, |_| {}, ApprovalStatus::Pending, WorkStatus::Normal).await;

    let response = gate_verify(&app, &terminal_token, common::WORKER_NFC_UID).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], false);
    assert!(
        body["reason"].as_str().unwrap().contains("승인 대기"),
        "차단 사유에 승인 대기가 포함돼야 한다: {}",
        body["reason"]
    );
}

/// AC-9 — 작업중지(STOPPED)는 승인 예외(APPROVED)로도 면제되지 않으며 사유에 "작업중지"가 있다
#[tokio::test]
async fn work_stopped_blocks_entry_even_when_approved() {
    let app = spawn_gate_app().await;
    let terminal_token = common::seed_active_terminal(&app).await;
    seed_attendance_with(&app, |_| {}, ApprovalStatus::Approved, WorkStatus::Stopped).await;

    let response = gate_verify(&app, &terminal_token, common::WORKER_NFC_UID).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json();
    assert_eq!(body["allowed"], false);
    assert!(
        body["reason"].as_str().unwrap().contains("작업중지"),
        "차단 사유에 작업중지가 포함돼야 한다: {}",
        body["reason"]
    );
}

async fn spawn_gate_app() -> TestApp {
    common::spawn_app_with(|router| router).await
}

async fn gate_verify(
    app: &TestApp,
    terminal_token: &str,
    nfc_card_uid: &str,
) -> common::MockResponse {
    app.verify_gate_with(Some(&format!("Bearer {terminal_token}")), nfc_card_uid)
        .await
}

async fn today_attendance(app: &TestApp) -> Option<attendances::Model> {
    attendances::Entity::find()
        .one(&app.db)
        .await
        .expect("출근 조회 실패")
}

/// 공통 픽스처로 만든 출근 건을 필요한 조각만 바꿔 시나리오 상태로 재구성한다.
async fn seed_attendance_with(
    app: &TestApp,
    mutate: impl FnOnce(&mut attendances::ActiveModel),
    approval_status: ApprovalStatus,
    work_status: WorkStatus,
) {
    common::spawn_worker(&app.db, common::WORKER_NFC_UID).await;
    let attendance = today_attendance(app)
        .await
        .expect("출근 픽스처가 존재해야 한다");
    let mut active: attendances::ActiveModel = attendance.into();
    mutate(&mut active);
    active.approval_status = Set(approval_status);
    active.work_status = Set(work_status);
    active
        .update(&app.db)
        .await
        .expect("출근 시나리오 재구성 실패");
}
