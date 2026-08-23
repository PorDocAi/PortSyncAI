use sea_orm::{ConnectionTrait, DbBackend, DbErr, EntityTrait, FromQueryResult, Statement, Value};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::models::cargo_document_versions::Entity as CargoDocumentVersions;
use crate::models::equipment_types::Entity as EquipmentTypes;
use crate::models::ppe_requirements::Entity as PpeRequirements;
use crate::models::work_ppe_requirement_snapshots::Entity as WorkPpeRequirementSnapshots;
use crate::models::works::Entity as Works;
use crate::utils::{
    AppState,
    auth::{AdminUser, AuthUser},
};

/// The generated PPE entities call this column `source_document_version_number`,
/// while migration 0005 creates the physical column as `source_document_version`.
/// Keep that compatibility detail local to this route until the schema generator is
/// corrected; the API and all other sidecar entities use the generated contract.
fn ppe_requirement_select(backend: DbBackend) -> String {
    let review_status = if backend == DbBackend::Postgres {
        "review_status::text"
    } else {
        "review_status"
    };
    format!(
        "SELECT ppe_requirement_id, equipment_type_id, category, performance_criteria, source_text, source_document_version_id, source_document_version AS source_document_version_number, {review_status} AS review_status, reviewed_by_id, reviewed_at, is_active FROM ppe_requirements"
    )
}

fn ppe_snapshot_select() -> &'static str {
    "SELECT work_ppe_requirement_snapshot_id, work_id, ppe_requirement_id, equipment_type_id, category, performance_criteria, source_text, source_document_version_id, source_document_version AS source_document_version_number, reviewed_by_id, reviewed_at, snapshotted_at FROM work_ppe_requirement_snapshots"
}

#[derive(Serialize, vespera::Schema)]
pub struct PpeErrorResponse {
    pub code: String,
    pub message: String,
}

fn ppe_error(
    status: StatusCode,
    code: &str,
    message: &str,
) -> (StatusCode, Json<PpeErrorResponse>) {
    (
        status,
        Json(PpeErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
}

fn internal_error() -> (StatusCode, Json<PpeErrorResponse>) {
    ppe_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "서버 오류가 발생했습니다.",
    )
}

fn placeholders(backend: DbBackend, count: usize) -> String {
    match backend {
        DbBackend::Postgres => (1..=count)
            .map(|index| format!("${index}"))
            .collect::<Vec<_>>()
            .join(", "),
        _ => std::iter::repeat_n("?", count)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn id_value(id: i64) -> Value {
    Value::BigInt(Some(id))
}

fn string_value(value: impl Into<String>) -> Value {
    Value::String(Some(value.into()))
}

fn optional_json_value(value: Option<serde_json::Value>) -> Value {
    Value::Json(value.map(Box::new))
}

fn timestamp_value(value: chrono::DateTime<chrono::FixedOffset>) -> Value {
    Value::ChronoDateTimeWithTimeZone(Some(value))
}

fn db_error(error: DbErr) -> (StatusCode, Json<PpeErrorResponse>) {
    eprintln!("PPE sidecar database error: {error:?}");
    internal_error()
}

#[derive(Clone, FromQueryResult)]
struct PpeRequirementRecord {
    ppe_requirement_id: i64,
    equipment_type_id: i64,
    category: String,
    performance_criteria: Option<serde_json::Value>,
    source_text: String,
    source_document_version_id: i64,
    source_document_version_number: i32,
    review_status: String,
    reviewed_by_id: Option<i64>,
    reviewed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    is_active: bool,
}

#[derive(Clone, FromQueryResult)]
struct PpeSnapshotRecord {
    work_ppe_requirement_snapshot_id: i64,
    work_id: i64,
    ppe_requirement_id: i64,
    equipment_type_id: i64,
    category: String,
    performance_criteria: Option<serde_json::Value>,
    source_text: String,
    source_document_version_id: i64,
    source_document_version_number: i32,
    reviewed_by_id: i64,
    reviewed_at: chrono::DateTime<chrono::FixedOffset>,
    snapshotted_at: chrono::DateTime<chrono::FixedOffset>,
}

async fn fetch_ppe_requirement(
    db: &sea_orm::DatabaseConnection,
    id: i64,
) -> Result<Option<PpeRequirementRecord>, DbErr> {
    let backend = db.get_database_backend();
    let sql = format!(
        "{} WHERE ppe_requirement_id = ?",
        ppe_requirement_select(backend)
    );
    let sql = if backend == DbBackend::Postgres {
        sql.replace('?', "$1")
    } else {
        sql
    };
    PpeRequirements::find()
        .from_raw_sql(Statement::from_sql_and_values(backend, sql, [id_value(id)]))
        .into_model::<PpeRequirementRecord>()
        .one(db)
        .await
}

async fn fetch_ppe_snapshot(
    db: &sea_orm::DatabaseConnection,
    id: i64,
) -> Result<Option<PpeSnapshotRecord>, DbErr> {
    let backend = db.get_database_backend();
    let sql = format!(
        "{} WHERE work_ppe_requirement_snapshot_id = ?",
        ppe_snapshot_select()
    );
    let sql = if backend == DbBackend::Postgres {
        sql.replace('?', "$1")
    } else {
        sql
    };
    WorkPpeRequirementSnapshots::find()
        .from_raw_sql(Statement::from_sql_and_values(backend, sql, [id_value(id)]))
        .into_model::<PpeSnapshotRecord>()
        .one(db)
        .await
}

#[derive(Serialize, vespera::Schema)]
pub struct PpeRequirementResponse {
    pub ppe_requirement_id: i64,
    pub equipment_type_id: i64,
    pub category: String,
    pub performance_criteria: Option<serde_json::Value>,
    pub source_text: String,
    pub source_document_version_id: i64,
    pub source_document_version_number: i32,
    pub review_status: String,
    pub reviewed_by_id: Option<i64>,
    pub reviewed_at: Option<String>,
    pub is_active: bool,
}

impl From<PpeRequirementRecord> for PpeRequirementResponse {
    fn from(m: PpeRequirementRecord) -> Self {
        Self {
            ppe_requirement_id: m.ppe_requirement_id,
            equipment_type_id: m.equipment_type_id,
            category: m.category,
            performance_criteria: m.performance_criteria,
            source_text: m.source_text,
            source_document_version_id: m.source_document_version_id,
            source_document_version_number: m.source_document_version_number,
            review_status: m.review_status,
            reviewed_by_id: m.reviewed_by_id,
            reviewed_at: m.reviewed_at.map(|ts| ts.to_rfc3339()),
            is_active: m.is_active,
        }
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct CreatePpeRequirementRequest {
    pub equipment_type_id: i64,
    pub category: String,
    pub performance_criteria: Option<serde_json::Value>,
    pub source_text: String,
    pub source_document_version_id: i64,
}

#[derive(Serialize, vespera::Schema)]
pub struct PpeSnapshotResponse {
    pub work_ppe_requirement_snapshot_id: i64,
    pub work_id: i64,
    pub ppe_requirement_id: i64,
    pub equipment_type_id: i64,
    pub category: String,
    pub performance_criteria: Option<serde_json::Value>,
    pub source_text: String,
    pub source_document_version_id: i64,
    pub source_document_version_number: i32,
    pub reviewed_by_id: i64,
    pub reviewed_at: String,
    pub snapshotted_at: String,
}

impl From<PpeSnapshotRecord> for PpeSnapshotResponse {
    fn from(m: PpeSnapshotRecord) -> Self {
        Self {
            work_ppe_requirement_snapshot_id: m.work_ppe_requirement_snapshot_id,
            work_id: m.work_id,
            ppe_requirement_id: m.ppe_requirement_id,
            equipment_type_id: m.equipment_type_id,
            category: m.category,
            performance_criteria: m.performance_criteria,
            source_text: m.source_text,
            source_document_version_id: m.source_document_version_id,
            source_document_version_number: m.source_document_version_number,
            reviewed_by_id: m.reviewed_by_id,
            reviewed_at: m.reviewed_at.to_rfc3339(),
            snapshotted_at: m.snapshotted_at.to_rfc3339(),
        }
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct CreatePpeSnapshotRequest {
    pub work_id: i64,
    pub ppe_requirement_id: i64,
}

#[derive(Deserialize, vespera::Schema)]
pub struct PpeSnapshotListQuery {
    pub work_id: Option<i64>,
}

/// 보호구 요구조건 목록
#[vespera::route(get, path = "/requirements", tags = ["ppe"])]
pub async fn list_ppe_requirements(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<PpeRequirementResponse>>, StatusCode> {
    let backend = state.db.get_database_backend();
    let rows = PpeRequirements::find()
        .from_raw_sql(Statement::from_string(
            backend,
            ppe_requirement_select(backend),
        ))
        .into_model::<PpeRequirementRecord>()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        rows.into_iter().map(PpeRequirementResponse::from).collect(),
    ))
}

/// 보호구 요구조건 생성 (관리 권한). 검수 전 PENDING 으로 저장한다.
#[vespera::route(post, path = "/requirements", tags = ["ppe"])]
pub async fn create_ppe_requirement(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreatePpeRequirementRequest>,
) -> Result<(StatusCode, Json<PpeRequirementResponse>), (StatusCode, Json<PpeErrorResponse>)> {
    EquipmentTypes::find_by_id(req.equipment_type_id)
        .one(&state.db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| {
            ppe_error(
                StatusCode::NOT_FOUND,
                "EQUIPMENT_TYPE_NOT_FOUND",
                "장비 종류를 찾을 수 없습니다.",
            )
        })?;
    let version = CargoDocumentVersions::find_by_id(req.source_document_version_id)
        .one(&state.db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| {
            ppe_error(
                StatusCode::NOT_FOUND,
                "DOCUMENT_VERSION_NOT_FOUND",
                "근거 문서 버전을 찾을 수 없습니다.",
            )
        })?;

    let backend = state.db.get_database_backend();
    let parameter_names = placeholders(backend, 8);
    let parameter_names = parameter_names.split(", ").collect::<Vec<_>>();
    let sql = format!(
        "INSERT INTO ppe_requirements (equipment_type_id, category, performance_criteria, source_text, source_document_version_id, source_document_version, review_status, reviewed_by_id, reviewed_at, is_active) VALUES ({}, {}, {}, {}, {}, {}, 'PENDING', {}, {}, true) RETURNING ppe_requirement_id",
        parameter_names[0],
        parameter_names[1],
        parameter_names[2],
        parameter_names[3],
        parameter_names[4],
        parameter_names[5],
        parameter_names[6],
        parameter_names[7],
    );
    let row = state
        .db
        .query_one_raw(Statement::from_sql_and_values(
            backend,
            sql,
            [
                id_value(req.equipment_type_id),
                string_value(req.category),
                optional_json_value(req.performance_criteria),
                string_value(req.source_text),
                id_value(req.source_document_version_id),
                Value::Int(Some(version.document_version)),
                Value::BigInt(None),
                Value::ChronoDateTimeWithTimeZone(None),
                Value::Bool(Some(true)),
            ],
        ))
        .await
        .map_err(db_error)?
        .ok_or_else(internal_error)?;
    let id: i64 = row.try_get("", "ppe_requirement_id").map_err(db_error)?;
    let saved = fetch_ppe_requirement(&state.db, id)
        .await
        .map_err(db_error)?
        .ok_or_else(internal_error)?;
    Ok((
        StatusCode::CREATED,
        Json(PpeRequirementResponse::from(saved)),
    ))
}

/// 보호구 요구조건 검수 확정 (관리 권한)
#[vespera::route(post, path = "/requirements/{id}/confirm", tags = ["ppe"])]
pub async fn confirm_ppe_requirement(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<PpeRequirementResponse>, (StatusCode, Json<PpeErrorResponse>)> {
    let requirement = fetch_ppe_requirement(&state.db, id)
        .await
        .map_err(db_error)?
        .ok_or_else(|| {
            ppe_error(
                StatusCode::NOT_FOUND,
                "PPE_REQUIREMENT_NOT_FOUND",
                "보호구 요구조건을 찾을 수 없습니다.",
            )
        })?;
    if requirement.review_status == "CONFIRMED" {
        return Ok(Json(PpeRequirementResponse::from(requirement)));
    }

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let backend = state.db.get_database_backend();
    let parameter_names = placeholders(backend, 4);
    let parameter_names = parameter_names.split(", ").collect::<Vec<_>>();
    let sql = format!(
        "UPDATE ppe_requirements SET review_status = 'CONFIRMED', reviewed_by_id = {}, reviewed_at = {}, updated_at = {} WHERE ppe_requirement_id = {}",
        parameter_names[0], parameter_names[1], parameter_names[2], parameter_names[3]
    );
    state
        .db
        .execute_raw(Statement::from_sql_and_values(
            backend,
            sql,
            [
                id_value(claims.sub),
                timestamp_value(now),
                timestamp_value(now),
                id_value(id),
            ],
        ))
        .await
        .map_err(db_error)?;
    let saved = fetch_ppe_requirement(&state.db, id)
        .await
        .map_err(db_error)?
        .ok_or_else(internal_error)?;
    Ok(Json(PpeRequirementResponse::from(saved)))
}

/// 작업 확정 시점의 불변 보호구 스냅샷 목록
#[vespera::route(get, path = "/snapshots", tags = ["ppe"])]
pub async fn list_ppe_snapshots(
    _auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<PpeSnapshotListQuery>,
) -> Result<Json<Vec<PpeSnapshotResponse>>, StatusCode> {
    let backend = state.db.get_database_backend();
    let (sql, values) = if let Some(work_id) = q.work_id {
        let sql = if backend == DbBackend::Postgres {
            format!(
                "{} WHERE work_id = $1 ORDER BY work_ppe_requirement_snapshot_id DESC",
                ppe_snapshot_select()
            )
        } else {
            format!(
                "{} WHERE work_id = ? ORDER BY work_ppe_requirement_snapshot_id DESC",
                ppe_snapshot_select()
            )
        };
        (sql, vec![id_value(work_id)])
    } else {
        (
            format!(
                "{} ORDER BY work_ppe_requirement_snapshot_id DESC",
                ppe_snapshot_select()
            ),
            Vec::new(),
        )
    };
    let rows = WorkPpeRequirementSnapshots::find()
        .from_raw_sql(Statement::from_sql_and_values(backend, sql, values))
        .into_model::<PpeSnapshotRecord>()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        rows.into_iter().map(PpeSnapshotResponse::from).collect(),
    ))
}

pub async fn snapshot_ids_for_work(
    db: &sea_orm::DatabaseConnection,
    work_id: i64,
) -> Result<Vec<i64>, DbErr> {
    let backend = db.get_database_backend();
    let sql = if backend == DbBackend::Postgres {
        format!(
            "{} WHERE work_id = $1 ORDER BY work_ppe_requirement_snapshot_id ASC",
            ppe_snapshot_select()
        )
    } else {
        format!(
            "{} WHERE work_id = ? ORDER BY work_ppe_requirement_snapshot_id ASC",
            ppe_snapshot_select()
        )
    };
    let rows = WorkPpeRequirementSnapshots::find()
        .from_raw_sql(Statement::from_sql_and_values(
            backend,
            sql,
            [id_value(work_id)],
        ))
        .into_model::<PpeSnapshotRecord>()
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| row.work_ppe_requirement_snapshot_id)
        .collect())
}

/// 검수 확정된 요구조건을 작업 스냅샷으로 고정한다.
#[vespera::route(post, path = "/snapshots", tags = ["ppe"])]
pub async fn create_ppe_snapshot(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreatePpeSnapshotRequest>,
) -> Result<(StatusCode, Json<PpeSnapshotResponse>), (StatusCode, Json<PpeErrorResponse>)> {
    Works::find_by_id(req.work_id)
        .one(&state.db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| {
            ppe_error(
                StatusCode::NOT_FOUND,
                "WORK_NOT_FOUND",
                "작업을 찾을 수 없습니다.",
            )
        })?;
    let requirement = fetch_ppe_requirement(&state.db, req.ppe_requirement_id)
        .await
        .map_err(db_error)?
        .ok_or_else(|| {
            ppe_error(
                StatusCode::NOT_FOUND,
                "PPE_REQUIREMENT_NOT_FOUND",
                "보호구 요구조건을 찾을 수 없습니다.",
            )
        })?;
    if requirement.review_status != "CONFIRMED" {
        return Err(ppe_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "PPE_REVIEW_NOT_CONFIRMED",
            "검수가 확정되지 않은 보호구 요구조건은 스냅샷할 수 없습니다.",
        ));
    }
    if !requirement.is_active {
        return Err(ppe_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "PPE_REQUIREMENT_INACTIVE",
            "비활성 보호구 요구조건은 스냅샷할 수 없습니다.",
        ));
    }
    let reviewed_by_id = requirement.reviewed_by_id.ok_or_else(|| {
        ppe_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "PPE_REVIEW_NOT_CONFIRMED",
            "검수자가 없는 보호구 요구조건은 스냅샷할 수 없습니다.",
        )
    })?;
    let reviewed_at = requirement.reviewed_at.ok_or_else(|| {
        ppe_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "PPE_REVIEW_NOT_CONFIRMED",
            "검수 시각이 없는 보호구 요구조건은 스냅샷할 수 없습니다.",
        )
    })?;

    let backend = state.db.get_database_backend();
    let sql = format!(
        "INSERT INTO work_ppe_requirement_snapshots (work_id, ppe_requirement_id, equipment_type_id, category, performance_criteria, source_text, source_document_version_id, source_document_version, reviewed_by_id, reviewed_at) VALUES ({}) RETURNING work_ppe_requirement_snapshot_id",
        placeholders(backend, 10)
    );
    let row = state
        .db
        .query_one_raw(Statement::from_sql_and_values(
            backend,
            sql,
            [
                id_value(req.work_id),
                id_value(requirement.ppe_requirement_id),
                id_value(requirement.equipment_type_id),
                string_value(requirement.category),
                optional_json_value(requirement.performance_criteria),
                string_value(requirement.source_text),
                id_value(requirement.source_document_version_id),
                Value::Int(Some(requirement.source_document_version_number)),
                id_value(reviewed_by_id),
                timestamp_value(reviewed_at),
            ],
        ))
        .await
        .map_err(|_| {
            ppe_error(
                StatusCode::CONFLICT,
                "PPE_SNAPSHOT_EXISTS",
                "해당 작업에 동일한 보호구 스냅샷이 이미 있습니다.",
            )
        })?
        .ok_or_else(internal_error)?;
    let id: i64 = row
        .try_get("", "work_ppe_requirement_snapshot_id")
        .map_err(db_error)?;
    let saved = fetch_ppe_snapshot(&state.db, id)
        .await
        .map_err(db_error)?
        .ok_or_else(internal_error)?;
    Ok((StatusCode::CREATED, Json(PpeSnapshotResponse::from(saved))))
}
