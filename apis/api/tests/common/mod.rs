//! 통합 테스트 공통 하네스스 (AC-1·2·10, #42)
//!
//! 외부 프로세스·포트 없이 in-process로 앱을 조립한다:
//! 임시 SQLite 파일 DB → vespertide 마이그레이션 전체 적용 →
//! 실제 `#[vespera::route]` 핸들러를 Router에 연결 → `tower::ServiceExt::oneshot` 요청.
//! 타이밍 의존 요소(sleep, 폴링)는 일절 없다 — 모든 단언은 응답 상태/바디로만 한다.
//
// 각 통합 테스트 바이너리가 이 모듈을 개별 컴파일하므로, 특정 바이너리가 쓰지 않는
// 공용 픽스처가 dead-code로 보인다 — 공용 하네스스 특성상 모듈 단위로 허용한다.
#![allow(dead_code)]

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ConnectionTrait, DatabaseConnection, DbBackend,
    EntityTrait, QueryResult, Set, Statement, Value,
};
use vespera::axum::{Router, http::StatusCode};

use api::config::Config;
use api::models::{
    attendances::{self, ApprovalStatus},
    cargo_document_versions::{
        self, CargoProcessingStatus, CargoReviewStatus, V2CargoDocumentType,
    },
    departments,
    employees::{self, SystemRole},
    equipment,
    equipment_profiles::{self, EquipmentLifecycleStatus, EquipmentOwnershipType},
    equipment_tag_tokens, equipment_types, job_roles,
    v2_work_assignments::{self, V2WorkAssignmentStatus},
    work_types,
    works::{self, WorkLifecycleStatus},
};
use api::routes::{gate, work_assignments};
use api::utils::AppState;

pub const ADMIN_TOKEN: &str = "admin-token";
pub const TERMINAL_TOKEN: &str = "terminal-token";
pub const WORKER_NFC_UID: &str = "nfc-worker-0001";

/// 관리자용 JWT (utils::jwt 재사용 — 서명 검증까지 실제 경로로 통과)
pub fn admin_bearer() -> String {
    format!(
        "Bearer {}",
        api::utils::jwt::create_token(1, "ADMIN", &test_config().jwt_secret)
            .expect("JWT 생성 실패")
    )
}

/// 비관리 권한(WORKER) JWT — 403 경로 검증용
pub fn worker_bearer() -> String {
    bearer_for(2, "WORKER")
}

/// 임의 직원/권한 JWT — tag_equipment·create_work_assignment는 클레임 sub로 호출자를
/// 찾으므로, 픽스처로 만든 직원 ID를 담은 토큰이 필요하다.
pub fn bearer_for(employee_id: i64, role: &str) -> String {
    format!(
        "Bearer {}",
        api::utils::jwt::create_token(employee_id, role, &test_config().jwt_secret)
            .expect("JWT 생성 실패")
    )
}

/// 테스트용 Config — DATABASE_URL/JWT_SECRET을 환경변수가 아닌 직접 주입으로 고정해
/// 다른 테스트·로컬 설정의 영향을 받지 않는다.
pub struct TestApp {
    pub db: DatabaseConnection,
    pub app: Router,
    /// SQLite 임시 파일 정리용 (필드 유지 — drop 시 삭제)
    _db_file: Option<tempfile::TempDir>,
}

/// 앱 + 마이그레이션된 빈 DB를 프로세스 안에 만든다.
pub async fn spawn_app() -> TestApp {
    spawn_app_with(|router| router).await
}

/// 라우터 조립을 커스터마이즈한다 — 각 테스트 파일은 자신이 검증하는
/// 핸들러만 실제 `#[vespera::route]` 계약으로 연결한다.
pub async fn spawn_app_with(
    customize: impl FnOnce(Router<AppState>) -> Router<AppState>,
) -> TestApp {
    let dir = tempfile::tempdir().expect("임시 디렉터리 생성 실패");
    let database_url = format!("sqlite://{}/test.db?mode=rwc", dir.path().to_string_lossy());

    let db = sea_orm::Database::connect(&database_url)
        .await
        .expect("테스트 DB 연결 실패");
    vespertide::vespertide_migration!(&db)
        .await
        .expect("마이그레이션 실패");

    let mut config = test_config();
    config.database_url = database_url;
    let state = AppState {
        db: db.clone(),
        config,
    };

    // main.rs가 라우트 수집에 쓰는 vespera! 매크로 대신, 대상 핸들러들만
    // 동일한 Router 계약으로 직접 연결한다.
    let base = Router::new()
        .route(
            "/gate/terminals",
            vespera::axum::routing::post(gate::issue_gate_terminal),
        )
        .route(
            "/equipment-checks/tag-tokens",
            vespera::axum::routing::post(api::routes::equipment_checks::issue_equipment_tag_token),
        )
        .route(
            "/equipment-checks/tag-tokens/{tokenid}",
            vespera::axum::routing::delete(
                api::routes::equipment_checks::revoke_equipment_tag_token,
            ),
        )
        .route(
            "/equipment-checks/{equipment_profile_id}/tag-token",
            vespera::axum::routing::post(
                api::routes::equipment_checks::issue_equipment_tag_token_for_profile,
            )
            .delete(api::routes::equipment_checks::revoke_equipment_tag_token_for_profile),
        )
        .route(
            "/equipment-checks/tag-tokens/{equipment_profile_id}/rotate",
            vespera::axum::routing::post(api::routes::equipment_checks::rotate_equipment_tag_token),
        )
        .route(
            "/gate/terminals/{terminalid}",
            vespera::axum::routing::delete(gate::revoke_gate_terminal),
        );
    let app = customize(base)
        .route(
            "/equipment-checks",
            vespera::axum::routing::post(api::routes::equipment_checks::tag_equipment),
        )
        .route(
            "/gate/verify",
            vespera::axum::routing::post(gate::verify_gate),
        )
        .route(
            "/work-assignments",
            vespera::axum::routing::post(work_assignments::create_work_assignment),
        )
        .route(
            "/work-assignments/{id}",
            vespera::axum::routing::patch(work_assignments::update_work_assignment),
        )
        .with_state(state);

    TestApp {
        app,
        db,
        _db_file: Some(dir),
    }
}

/// AC-2·10 픽스처: 게이트 단말 1개와 그 토큰.
/// 발급 API(POST /gate/terminals)를 실제로 호출해 해시 저장 경로까지 함께 검증한다.
pub async fn seed_active_terminal(app: &TestApp) -> String {
    let response = app
        .send_request(
            req(vespera::axum::http::Method::POST, "/gate/terminals")
                .header("Authorization", admin_bearer())
                .header("Content-Type", "application/json")
                .body(serde_json::json!({ "gate_id": "북문" }).to_string())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response.json();
    body["token"]
        .as_str()
        .expect("발급 응답에 token 없음")
        .to_string()
}

/// AC-1 픽스처: 오늘자 출근 절차가 시작된 작업원 (안전지침 확인 + 장비 착용 완료)
pub async fn seed_worker_with_started_attendance(db: &DatabaseConnection) -> employees::Model {
    let (worker, _attendance) = spawn_worker(db, WORKER_NFC_UID).await;
    worker
}

/// 임의 사원증 UID로 직원 + 오늘자 출근(지침확인·장비완료·승인완료) 픽스처를 만든다.
/// 사번·이메일·부서·직무 코드는 UUID 접미사로 고유화해 테스트 간 충돌이 없게 한다.
pub async fn spawn_worker(
    db: &DatabaseConnection,
    nfc_card_uid: &str,
) -> (employees::Model, attendances::Model) {
    let unique = uuid::Uuid::new_v4().simple().to_string();

    let department = departments::ActiveModel {
        department_id: NotSet,
        department_code: Set(format!("DEP-{unique}")),
        name: Set("테스트부서".to_string()),
        description: Set(None),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(db)
    .await
    .expect("부서 픽스처 삽입 실패");

    let job_role = job_roles::ActiveModel {
        job_role_id: NotSet,
        job_role_code: Set(format!("ROLE-{unique}")),
        name: Set("테스트직무".to_string()),
        description: Set(None),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(db)
    .await
    .expect("직무 픽스처 삽입 실패");

    let worker = employees::ActiveModel {
        employee_id: NotSet,
        employee_number: Set(unique.clone()),
        name: Set("홍길동".to_string()),
        email: Set(format!("hong-{unique}@test.local")),
        password_hash: Set("test-hash".to_string()),
        phone_number: Set(None),
        department_id: Set(department.department_id),
        job_role_id: Set(job_role.job_role_id),
        position: Set(None),
        system_role: Set(SystemRole::Worker),
        preferred_language: Set("ko".to_string()),
        nfc_card_uid: Set(Some(nfc_card_uid.to_string())),
        nfc_card_issued_at: Set(None),
        hire_date: Set(api::routes::attendances::today()),
        status: Default::default(), // ACTIVE
        created_at: NotSet,
        updated_at: Set(None),
        resigned_at: Set(None),
    }
    .insert(db)
    .await
    .expect("작업원 픽스처 삽입 실패");

    let attendance = attendances::ActiveModel {
        attendance_id: NotSet,
        employee_id: Set(worker.employee_id),
        work_date: Set(api::routes::attendances::today()),
        instruction_ack_completed: Set(true),
        equipment_check_completed: Set(true),
        approval_status: Set(ApprovalStatus::Approved),
        gate_status: Default::default(), // BLOCKED
        work_status: Default::default(), // NORMAL
        gate_passed_at: Set(None),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(db)
    .await
    .expect("출근 픽스처 삽입 실패");

    (worker, attendance)
}

/// 활성 여부를 조회하는 편의 함수 (폐기 API가 소프트 삭제하는 것을 확인할 때 사용)
pub async fn terminal_is_active(db: &DatabaseConnection, terminal_id: i64) -> bool {
    api::models::gate_terminals::Entity::find_by_id(terminal_id)
        .one(db)
        .await
        .expect("단말 조회 실패")
        .expect("단말이 존재해야 한다")
        .is_active
}

/// 토큰 해시가 실제로 저장됐는지 확인 (SHA-256 해시만 저장 — 평문 저장 금지)
#[allow(dead_code)]
pub async fn terminal_token_hash(db: &DatabaseConnection, terminal_id: i64) -> String {
    api::models::gate_terminals::Entity::find_by_id(terminal_id)
        .one(db)
        .await
        .expect("단말 조회 실패")
        .expect("단말이 존재해야 한다")
        .token_hash
}

/// v2 장비 프로필 픽스처. legacy equipment 행은 v2 경로에서 참조하지 않는다.
pub async fn seed_v2_equipment(
    db: &DatabaseConnection,
    owner_employee_id: Option<i64>,
    ownership_type: EquipmentOwnershipType,
    status: EquipmentLifecycleStatus,
) -> equipment_profiles::Model {
    let type_id = match equipment_types::Entity::find()
        .one(db)
        .await
        .expect("장비 종류 조회 실패")
    {
        Some(existing) => existing.equipment_type_id,
        None => {
            equipment_types::ActiveModel {
                equipment_type_id: NotSet,
                name: Set("안전화".to_string()),
                category: Set(None),
                description: Set(None),
                created_at: NotSet,
                updated_at: Set(None),
            }
            .insert(db)
            .await
            .expect("장비 종류 픽스처 삽입 실패")
            .equipment_type_id
        }
    };
    equipment_profiles::ActiveModel {
        equipment_profile_id: NotSet,
        legacy_equipment_id: Set(None),
        equipment_type_id: Set(type_id),
        asset_number: Set(Some(format!("V2-AST-{}", uuid::Uuid::new_v4().simple()))),
        ownership_type: Set(ownership_type),
        owner_employee_id: Set(owner_employee_id),
        status: Set(status),
        status_reason: Set(None),
        braille_label: Set(None),
        manufacturer_replacement_due_at: Set(None),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(db)
    .await
    .expect("v2 장비 프로필 픽스처 삽입 실패")
}

pub async fn seed_v2_token(
    db: &DatabaseConnection,
    equipment_profile_id: i64,
    issued_by_id: i64,
) -> (equipment_tag_tokens::Model, String) {
    let token = format!("eqt_test_{}", uuid::Uuid::new_v4().simple());
    let saved = equipment_tag_tokens::ActiveModel {
        equipment_tag_token_id: NotSet,
        equipment_profile_id: Set(equipment_profile_id),
        tag_token_hash: Set(api::utils::auth::hash_token(&token)),
        is_active: Set(true),
        issued_by_id: Set(issued_by_id),
        issued_at: NotSet,
        deactivated_by_id: Set(None),
        deactivated_at: Set(None),
        created_at: NotSet,
    }
    .insert(db)
    .await
    .expect("v2 장비 토큰 픽스처 삽입 실패");
    (saved, token)
}

async fn insert_v2_ppe_requirement(
    db: &DatabaseConnection,
    equipment_type_id: i64,
    source_document_version_id: i64,
) -> i64 {
    let backend = db.get_database_backend();
    let placeholders = if backend == DbBackend::Postgres {
        "$1, $2, $3, $4, $5, $6, $7, $8, $9, $10"
    } else {
        "?, ?, ?, ?, ?, ?, ?, ?, ?, ?"
    };
    let sql = format!(
        "INSERT INTO ppe_requirements (equipment_type_id, category, performance_criteria, source_text, source_document_version_id, source_document_version, review_status, reviewed_by_id, reviewed_at, is_active) VALUES ({placeholders}) RETURNING ppe_requirement_id"
    );
    let row: QueryResult = db
        .query_one_raw(Statement::from_sql_and_values(
            backend,
            sql,
            vec![
                Value::BigInt(Some(equipment_type_id)),
                Value::String(Some("FOOT".to_string())),
                Value::Json(None),
                Value::String(Some("테스트 PPE 요구조건".to_string())),
                Value::BigInt(Some(source_document_version_id)),
                Value::Int(Some(1)),
                Value::String(Some("CONFIRMED".to_string())),
                Value::BigInt(Some(1)),
                Value::ChronoDateTimeWithTimeZone(Some(Utc::now().fixed_offset())),
                Value::Bool(Some(true)),
            ],
        ))
        .await
        .expect("PPE 요구조건 삽입 실패")
        .expect("PPE 요구조건 반환 실패");
    row.try_get("", "ppe_requirement_id")
        .expect("PPE 요구조건 ID 반환 실패")
}

async fn insert_v2_ppe_snapshot(
    db: &DatabaseConnection,
    work_id: i64,
    ppe_requirement_id: i64,
    equipment_type_id: i64,
    source_document_version_id: i64,
) -> i64 {
    let backend = db.get_database_backend();
    let placeholders = if backend == DbBackend::Postgres {
        "$1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11"
    } else {
        "?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?"
    };
    let sql = format!(
        "INSERT INTO work_ppe_requirement_snapshots (work_id, ppe_requirement_id, equipment_type_id, category, performance_criteria, source_text, source_document_version_id, source_document_version, reviewed_by_id, reviewed_at, snapshotted_at) VALUES ({placeholders}) RETURNING work_ppe_requirement_snapshot_id"
    );
    let row: QueryResult = db
        .query_one_raw(Statement::from_sql_and_values(
            backend,
            sql,
            vec![
                Value::BigInt(Some(work_id)),
                Value::BigInt(Some(ppe_requirement_id)),
                Value::BigInt(Some(equipment_type_id)),
                Value::String(Some("FOOT".to_string())),
                Value::Json(None),
                Value::String(Some("테스트 PPE 스냅샷".to_string())),
                Value::BigInt(Some(source_document_version_id)),
                Value::Int(Some(1)),
                Value::BigInt(Some(1)),
                Value::ChronoDateTimeWithTimeZone(Some(Utc::now().fixed_offset())),
                Value::ChronoDateTimeWithTimeZone(Some(Utc::now().fixed_offset())),
            ],
        ))
        .await
        .expect("PPE 스냅샷 삽입 실패")
        .expect("PPE 스냅샷 반환 실패");
    row.try_get("", "work_ppe_requirement_snapshot_id")
        .expect("PPE 스냅샷 ID 반환 실패")
}

/// v2 작업·선택된 배정·확정 PPE 스냅샷 픽스처.
pub async fn seed_v2_assignment(
    db: &DatabaseConnection,
    employee_id: i64,
    equipment_type_id: i64,
) -> v2_work_assignments::Model {
    let work_type = work_types::ActiveModel {
        work_type_id: NotSet,
        work_type_code: Set(format!("V2-WORK-{}", uuid::Uuid::new_v4().simple())),
        name: Set("테스트 작업".to_string()),
        description: Set(None),
        is_active: Set(true),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(db)
    .await
    .expect("작업 유형 픽스처 삽입 실패");
    let work = works::ActiveModel {
        work_id: NotSet,
        work_type_id: Set(work_type.work_type_id),
        work_reference: Set(format!("V2-REF-{}", uuid::Uuid::new_v4().simple())),
        status: Set(WorkLifecycleStatus::Active),
        scheduled_start_at: Set(Utc::now().fixed_offset()),
        scheduled_end_at: Set(None),
        started_at: Set(Some(Utc::now().fixed_offset())),
        completed_at: Set(None),
        created_by_id: Set(1),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(db)
    .await
    .expect("v2 작업 픽스처 삽입 실패");
    let document = cargo_document_versions::ActiveModel {
        cargo_document_version_id: NotSet,
        legacy_cargo_document_id: Set(None),
        document_type: Set(V2CargoDocumentType::Msds),
        document_version: Set(1),
        processing_status: Set(CargoProcessingStatus::Completed),
        review_status: Set(CargoReviewStatus::Confirmed),
        reviewed_by_id: Set(Some(1)),
        reviewed_at: Set(Some(Utc::now().fixed_offset())),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(db)
    .await
    .expect("v2 문서 픽스처 삽입 실패");
    let ppe =
        insert_v2_ppe_requirement(db, equipment_type_id, document.cargo_document_version_id).await;
    insert_v2_ppe_snapshot(
        db,
        work.work_id,
        ppe,
        equipment_type_id,
        document.cargo_document_version_id,
    )
    .await;
    v2_work_assignments::ActiveModel {
        work_assignment_id: NotSet,
        legacy_assignment_id: Set(None),
        work_id: Set(work.work_id),
        employee_id: Set(employee_id),
        assigned_by_id: Set(1),
        status: Set(V2WorkAssignmentStatus::Active),
        selected_at: Set(Some(Utc::now().fixed_offset())),
        started_at: Set(Some(Utc::now().fixed_offset())),
        completed_at: Set(None),
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(db)
    .await
    .expect("v2 배정 픽스처 삽입 실패")
}

/// AC-6 legacy 픽스처는 다른 테스트(현행 legacy API)에서만 사용한다.
pub async fn seed_active_equipment(db: &DatabaseConnection) -> equipment::Model {
    let type_id = match equipment_types::Entity::find()
        .one(db)
        .await
        .expect("장비 종류 조회 실패")
    {
        Some(existing) => existing.equipment_type_id,
        None => {
            equipment_types::ActiveModel {
                equipment_type_id: NotSet,
                name: Set("안전화".to_string()),
                category: Set(None),
                description: Set(None),
                created_at: NotSet,
                updated_at: Set(None),
            }
            .insert(db)
            .await
            .expect("장비 종류 픽스처 삽입 실패")
            .equipment_type_id
        }
    };

    equipment::ActiveModel {
        equipment_id: NotSet,
        equipment_type_id: Set(type_id),
        asset_number: Set(Some(format!("AST-{}", uuid::Uuid::new_v4().simple()))),
        nfc_tag_uid: Set(format!("eq-{}", uuid::Uuid::new_v4().simple())),
        braille_label: Set(None),
        is_active: Default::default(), // true
        created_at: NotSet,
        updated_at: Set(None),
    }
    .insert(db)
    .await
    .expect("장비 픽스처 삽입 실패")
}

impl TestApp {
    /// Router에 oneshot 요청을 보낸다. sleep·포트 바인딩 없이 즉시 응답을 받는다.
    pub async fn send_request(
        &self,
        request: vespera::axum::http::Request<String>,
    ) -> MockResponse {
        use tower::util::ServiceExt;
        let response = self
            .app
            .clone()
            .oneshot(request)
            .await
            .expect("요청 처리 실패");
        MockResponse {
            status: response.status(),
            body: to_bytes(response.into_body()).await,
        }
    }

    pub async fn tag_equipment_with(
        &self,
        authorization: &str,
        work_assignment_id: i64,
        tag_token: &str,
        idempotency_key: &str,
    ) -> MockResponse {
        self.send_request(
            req(vespera::axum::http::Method::POST, "/equipment-checks")
                .header("Authorization", authorization)
                .header("Content-Type", "application/json")
                .body(
                    serde_json::json!({
                        "work_assignment_id": work_assignment_id,
                        "tag_token": tag_token,
                        "client_scanned_at": "2026-08-23T09:00:00+09:00",
                        "idempotency_key": idempotency_key,
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
    }

    pub async fn issue_equipment_tag_token_with(
        &self,
        authorization: &str,
        equipment_profile_id: i64,
    ) -> MockResponse {
        self.send_request(
            req(
                vespera::axum::http::Method::POST,
                "/equipment-checks/tag-tokens",
            )
            .header("Authorization", authorization)
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "equipment_profile_id": equipment_profile_id }).to_string())
            .unwrap(),
        )
        .await
    }

    pub async fn revoke_equipment_tag_token_with(
        &self,
        authorization: &str,
        token_id: i64,
    ) -> MockResponse {
        self.send_request(
            req(
                vespera::axum::http::Method::DELETE,
                &format!("/equipment-checks/tag-tokens/{token_id}"),
            )
            .header("Authorization", authorization)
            .body(String::new())
            .unwrap(),
        )
        .await
    }

    pub async fn create_assignment_with(
        &self,
        authorization: &str,
        employee_id: i64,
        cargo_item_id: Option<i64>,
    ) -> MockResponse {
        self.send_request(
            req(vespera::axum::http::Method::POST, "/work-assignments")
                .header("Authorization", authorization)
                .header("Content-Type", "application/json")
                .body(
                    serde_json::json!({
                        "employee_id": employee_id,
                        "cargo_item_id": cargo_item_id,
                    })
                    .to_string(),
                )
                .unwrap(),
        )
        .await
    }

    pub async fn update_assignment_with(
        &self,
        authorization: &str,
        assignment_id: i64,
        cargo_item_id: Option<i64>,
    ) -> MockResponse {
        self.send_request(
            req(
                vespera::axum::http::Method::PATCH,
                &format!("/work-assignments/{assignment_id}"),
            )
            .header("Authorization", authorization)
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "cargo_item_id": cargo_item_id }).to_string())
            .unwrap(),
        )
        .await
    }

    pub async fn verify_gate_with(
        &self,
        authorization: Option<&str>,
        nfc_card_uid: &str,
    ) -> MockResponse {
        let mut request = req(vespera::axum::http::Method::POST, "/gate/verify")
            .header("Content-Type", "application/json");
        if let Some(token) = authorization {
            request = request.header("Authorization", token);
        }
        let body = if nfc_card_uid.trim_start().starts_with('{') {
            nfc_card_uid.to_string()
        } else {
            serde_json::json!({ "nfc_card_uid": nfc_card_uid }).to_string()
        };
        self.send_request(request.body(body).unwrap()).await
    }

    pub async fn revoke_terminal_with(
        &self,
        authorization: &str,
        terminal_id: i64,
    ) -> MockResponse {
        self.send_request(
            req(
                vespera::axum::http::Method::DELETE,
                &format!("/gate/terminals/{terminal_id}"),
            )
            .header("Authorization", authorization)
            .body(String::new())
            .unwrap(),
        )
        .await
    }
}

fn req(method: vespera::axum::http::Method, uri: &str) -> vespera::axum::http::request::Builder {
    vespera::axum::http::Request::builder()
        .method(method)
        .uri(uri)
}

pub struct MockResponse {
    status: StatusCode,
    body: Vec<u8>,
}

impl MockResponse {
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).expect("응답 바디 JSON 파싱 실패")
    }
}

async fn to_bytes(body: vespera::axum::body::Body) -> Vec<u8> {
    use http_body_util::BodyExt;
    BodyExt::collect(body)
        .await
        .expect("바디 수집 실패")
        .to_bytes()
        .into()
}

/// 테스트 JWT 시크릿 — 위조 토큰(다른 시크릿 서명)을 만들 때 사용한다.
pub(crate) fn jwt_secret() -> String {
    test_config().jwt_secret
}

fn test_config() -> Config {
    Config {
        database_url: String::new(),
        jwt_secret: "integration-test-secret".to_string(),
        upload_dir: std::env::temp_dir()
            .join("portdocai-test-uploads")
            .to_string_lossy()
            .into_owned(),
        port: 0,
    }
}
