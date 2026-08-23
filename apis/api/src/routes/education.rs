use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

use crate::models::cargo_items::{self, Entity as CargoItems};
use crate::models::dg_classes::Entity as DgClasses;
use crate::models::education_completions::{self, Entity as EducationCompletions};
use crate::models::education_courses::{self, Entity as EducationCourses};
use crate::models::education_target_rules::{self, Entity as EducationTargetRules};
use crate::models::employees::Entity as Employees;
use crate::models::work_targets::{self, Entity as WorkTargets};
use crate::models::work_types::Entity as WorkTypes;
use crate::models::works::Entity as Works;
use crate::utils::{
    AppState,
    auth::{AdminUser, AuthUser},
};

#[derive(Serialize, vespera::Schema)]
pub struct EducationCourseResponse {
    pub education_course_id: i64,
    pub course_code: String,
    pub title: String,
    pub description: Option<String>,
    pub version: i32,
    pub validity_days: Option<i32>,
    pub is_active: bool,
}

impl From<education_courses::Model> for EducationCourseResponse {
    fn from(m: education_courses::Model) -> Self {
        Self {
            education_course_id: m.education_course_id,
            course_code: m.course_code,
            title: m.title,
            description: m.description,
            version: m.version,
            validity_days: m.validity_days,
            is_active: m.is_active,
        }
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct CreateEducationCourseRequest {
    pub course_code: String,
    pub title: String,
    pub description: Option<String>,
    pub version: Option<i32>,
    pub validity_days: Option<i32>,
}

#[derive(Serialize, vespera::Schema)]
pub struct EducationTargetRuleResponse {
    pub education_target_rule_id: i64,
    pub education_course_id: i64,
    pub work_type_id: Option<i64>,
    pub dg_class_id: Option<i64>,
    pub is_required: bool,
    pub is_active: bool,
}

impl From<education_target_rules::Model> for EducationTargetRuleResponse {
    fn from(m: education_target_rules::Model) -> Self {
        Self {
            education_target_rule_id: m.education_target_rule_id,
            education_course_id: m.education_course_id,
            work_type_id: m.work_type_id,
            dg_class_id: m.dg_class_id,
            is_required: m.is_required,
            is_active: m.is_active,
        }
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct CreateEducationTargetRuleRequest {
    pub education_course_id: i64,
    pub work_type_id: Option<i64>,
    pub dg_class_id: Option<i64>,
    pub is_required: Option<bool>,
}

#[derive(Serialize, vespera::Schema)]
pub struct EducationCompletionResponse {
    pub education_completion_id: i64,
    pub education_course_id: i64,
    pub employee_id: i64,
    pub course_version: i32,
    pub completed_at: String,
    pub expires_at: Option<String>,
    pub evidence_url: Option<String>,
    pub recorded_by_id: i64,
    pub fulfilled: bool,
}

#[derive(Deserialize, vespera::Schema)]
pub struct CreateEducationCompletionRequest {
    pub education_course_id: i64,
    pub employee_id: i64,
    /// RFC3339
    pub completed_at: String,
    /// RFC3339, 생략 시 과정의 validity_days로 계산
    pub expires_at: Option<String>,
    pub evidence_url: Option<String>,
    pub course_version: Option<i32>,
}

#[derive(Deserialize, vespera::Schema)]
pub struct EducationReadinessQuery {
    pub employee_id: i64,
    pub work_id: i64,
}

#[derive(Deserialize, vespera::Schema)]
pub struct EducationCompletionQuery {
    pub employee_id: Option<i64>,
    pub education_course_id: Option<i64>,
}

#[derive(Serialize, vespera::Schema)]
pub struct EducationRequirementStatus {
    pub education_course_id: i64,
    pub course_code: String,
    pub title: String,
    /// CURRENT | EXPIRED | MISSING
    pub status: String,
    pub fulfilled: bool,
    pub education_completion_id: Option<i64>,
    pub expires_at: Option<String>,
}

#[derive(Serialize, vespera::Schema)]
pub struct EducationReadinessResponse {
    pub employee_id: i64,
    pub work_id: i64,
    pub fulfilled: bool,
    pub requirements: Vec<EducationRequirementStatus>,
}

#[derive(Serialize, vespera::Schema)]
pub struct EducationErrorResponse {
    pub code: String,
    pub message: String,
}

fn education_error(
    status: StatusCode,
    code: &str,
    message: &str,
) -> (StatusCode, Json<EducationErrorResponse>) {
    (
        status,
        Json(EducationErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
}

fn internal_error() -> (StatusCode, Json<EducationErrorResponse>) {
    education_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "서버 오류가 발생했습니다.",
    )
}

fn parse_rfc3339(
    value: &str,
) -> Result<chrono::DateTime<chrono::FixedOffset>, (StatusCode, Json<EducationErrorResponse>)> {
    chrono::DateTime::parse_from_rfc3339(value).map_err(|_| {
        education_error(
            StatusCode::BAD_REQUEST,
            "INVALID_TIMESTAMP",
            "시각은 RFC3339 형식이어야 합니다.",
        )
    })
}

fn completion_is_current(
    completed_at: chrono::DateTime<chrono::FixedOffset>,
    expires_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> bool {
    completed_at <= now && expires_at.is_none_or(|expires_at| expires_at > now)
}

fn completion_response(
    m: education_completions::Model,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> EducationCompletionResponse {
    EducationCompletionResponse {
        education_completion_id: m.education_completion_id,
        education_course_id: m.education_course_id,
        employee_id: m.employee_id,
        course_version: m.course_version,
        completed_at: m.completed_at.to_rfc3339(),
        expires_at: m.expires_at.map(|ts| ts.to_rfc3339()),
        evidence_url: m.evidence_url,
        recorded_by_id: m.recorded_by_id,
        fulfilled: completion_is_current(m.completed_at, m.expires_at, now),
    }
}

/// 교육 과정 목록
#[vespera::route(get, path = "/courses", tags = ["education"])]
pub async fn list_education_courses(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<EducationCourseResponse>>, StatusCode> {
    let rows = EducationCourses::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        rows.into_iter()
            .map(EducationCourseResponse::from)
            .collect(),
    ))
}

/// 교육 과정 생성 (관리 권한)
#[vespera::route(post, path = "/courses", tags = ["education"])]
pub async fn create_education_course(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateEducationCourseRequest>,
) -> Result<(StatusCode, Json<EducationCourseResponse>), StatusCode> {
    let course = education_courses::ActiveModel {
        course_code: Set(req.course_code),
        title: Set(req.title),
        description: Set(req.description),
        version: Set(req.version.unwrap_or(1)),
        validity_days: Set(req.validity_days),
        ..Default::default()
    };
    let saved = course
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::CONFLICT)?;
    Ok((
        StatusCode::CREATED,
        Json(EducationCourseResponse::from(saved)),
    ))
}

/// 교육 대상 규칙 목록
#[vespera::route(get, path = "/rules", tags = ["education"])]
pub async fn list_education_target_rules(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<EducationTargetRuleResponse>>, StatusCode> {
    let rows = EducationTargetRules::find()
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        rows.into_iter()
            .map(EducationTargetRuleResponse::from)
            .collect(),
    ))
}

/// 교육 대상 규칙 생성 (관리 권한)
#[vespera::route(post, path = "/rules", tags = ["education"])]
pub async fn create_education_target_rule(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateEducationTargetRuleRequest>,
) -> Result<
    (StatusCode, Json<EducationTargetRuleResponse>),
    (StatusCode, Json<EducationErrorResponse>),
> {
    EducationCourses::find_by_id(req.education_course_id)
        .one(&state.db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| {
            education_error(
                StatusCode::NOT_FOUND,
                "EDUCATION_COURSE_NOT_FOUND",
                "교육 과정을 찾을 수 없습니다.",
            )
        })?;
    if let Some(work_type_id) = req.work_type_id {
        WorkTypes::find_by_id(work_type_id)
            .one(&state.db)
            .await
            .map_err(|_| internal_error())?
            .ok_or_else(|| {
                education_error(
                    StatusCode::NOT_FOUND,
                    "WORK_TYPE_NOT_FOUND",
                    "작업 유형을 찾을 수 없습니다.",
                )
            })?;
    }
    if let Some(dg_class_id) = req.dg_class_id {
        DgClasses::find_by_id(dg_class_id)
            .one(&state.db)
            .await
            .map_err(|_| internal_error())?
            .ok_or_else(|| {
                education_error(
                    StatusCode::NOT_FOUND,
                    "DG_CLASS_NOT_FOUND",
                    "위험물 등급을 찾을 수 없습니다.",
                )
            })?;
    }

    let rule = education_target_rules::ActiveModel {
        education_course_id: Set(req.education_course_id),
        work_type_id: Set(req.work_type_id),
        dg_class_id: Set(req.dg_class_id),
        is_required: Set(req.is_required.unwrap_or(true)),
        ..Default::default()
    };
    let saved = rule.insert(&state.db).await.map_err(|_| internal_error())?;
    Ok((
        StatusCode::CREATED,
        Json(EducationTargetRuleResponse::from(saved)),
    ))
}

/// 교육 이수 목록. employee_id/course_id로 작업자의 이수 이력을 조회할 수 있다.
#[vespera::route(get, path = "/completions", tags = ["education"])]
pub async fn list_education_completions(
    _auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EducationCompletionQuery>,
) -> Result<Json<Vec<EducationCompletionResponse>>, StatusCode> {
    let mut query = EducationCompletions::find();
    if let Some(employee_id) = q.employee_id {
        query = query.filter(education_completions::Column::EmployeeId.eq(employee_id));
    }
    if let Some(course_id) = q.education_course_id {
        query = query.filter(education_completions::Column::EducationCourseId.eq(course_id));
    }
    let rows = query
        .order_by_desc(education_completions::Column::CompletedAt)
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    Ok(Json(
        rows.into_iter()
            .map(|row| completion_response(row, now))
            .collect(),
    ))
}

/// 교육 이수 등록 (관리 권한). 동일 과정 재이수를 허용한다.
#[vespera::route(post, path = "/completions", tags = ["education"])]
pub async fn create_education_completion(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateEducationCompletionRequest>,
) -> Result<
    (StatusCode, Json<EducationCompletionResponse>),
    (StatusCode, Json<EducationErrorResponse>),
> {
    let course = EducationCourses::find_by_id(req.education_course_id)
        .one(&state.db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| {
            education_error(
                StatusCode::NOT_FOUND,
                "EDUCATION_COURSE_NOT_FOUND",
                "교육 과정을 찾을 수 없습니다.",
            )
        })?;
    Employees::find_by_id(req.employee_id)
        .one(&state.db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| {
            education_error(
                StatusCode::NOT_FOUND,
                "EMPLOYEE_NOT_FOUND",
                "작업자를 찾을 수 없습니다.",
            )
        })?;

    let completed_at = parse_rfc3339(&req.completed_at)?;
    let expires_at = if let Some(expires_at) = req.expires_at {
        Some(parse_rfc3339(&expires_at)?)
    } else {
        course
            .validity_days
            .map(|days| completed_at + chrono::Duration::days(days as i64))
    };
    if expires_at.is_some_and(|expires_at| expires_at <= completed_at) {
        return Err(education_error(
            StatusCode::BAD_REQUEST,
            "INVALID_EXPIRY",
            "이수 만료 시각은 이수 시각 이후여야 합니다.",
        ));
    }
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let completion = education_completions::ActiveModel {
        education_course_id: Set(req.education_course_id),
        employee_id: Set(req.employee_id),
        course_version: Set(req.course_version.unwrap_or(course.version)),
        completed_at: Set(completed_at),
        expires_at: Set(expires_at),
        evidence_url: Set(req.evidence_url),
        recorded_by_id: Set(claims.sub),
        ..Default::default()
    };
    let saved = completion
        .insert(&state.db)
        .await
        .map_err(|_| internal_error())?;
    Ok((StatusCode::CREATED, Json(completion_response(saved, now))))
}

/// 작업 배정 전 교육 충족 여부. CURRENT / EXPIRED / MISSING 을 구분한다.
#[vespera::route(get, path = "/readiness", tags = ["education"])]
pub async fn get_education_readiness(
    _auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EducationReadinessQuery>,
) -> Result<Json<EducationReadinessResponse>, (StatusCode, Json<EducationErrorResponse>)> {
    evaluate_education_readiness(&state.db, q.employee_id, q.work_id)
        .await
        .map(Json)
}

pub async fn evaluate_education_readiness(
    db: &sea_orm::DatabaseConnection,
    employee_id: i64,
    work_id: i64,
) -> Result<EducationReadinessResponse, (StatusCode, Json<EducationErrorResponse>)> {
    let work = Works::find_by_id(work_id)
        .one(db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| {
            education_error(
                StatusCode::NOT_FOUND,
                "WORK_NOT_FOUND",
                "작업을 찾을 수 없습니다.",
            )
        })?;
    Employees::find_by_id(employee_id)
        .one(db)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(|| {
            education_error(
                StatusCode::NOT_FOUND,
                "EMPLOYEE_NOT_FOUND",
                "작업자를 찾을 수 없습니다.",
            )
        })?;

    let rules = EducationTargetRules::find()
        .filter(education_target_rules::Column::IsActive.eq(true))
        .filter(education_target_rules::Column::IsRequired.eq(true))
        .all(db)
        .await
        .map_err(|_| internal_error())?;
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let mut requirements = Vec::new();

    let targets = WorkTargets::find()
        .filter(work_targets::Column::WorkId.eq(work_id))
        .all(db)
        .await
        .map_err(|_| internal_error())?;
    let target_cargo_ids: Vec<i64> = targets
        .into_iter()
        .filter_map(|target| target.cargo_item_id)
        .collect();
    let target_cargo = if target_cargo_ids.is_empty() {
        Vec::new()
    } else {
        CargoItems::find()
            .filter(cargo_items::Column::CargoItemId.is_in(target_cargo_ids))
            .all(db)
            .await
            .map_err(|_| internal_error())?
    };

    for rule in rules {
        if let Some(work_type_id) = rule.work_type_id
            && work_type_id != work.work_type_id
        {
            continue;
        }
        if let Some(dg_class_id) = rule.dg_class_id
            && !target_cargo
                .iter()
                .any(|cargo| cargo.dg_class_id == Some(dg_class_id))
        {
            continue;
        }
        let course = EducationCourses::find_by_id(rule.education_course_id)
            .one(db)
            .await
            .map_err(|_| internal_error())?
            .ok_or_else(internal_error)?;
        if !course.is_active {
            continue;
        }
        let latest = EducationCompletions::find()
            .filter(education_completions::Column::EmployeeId.eq(employee_id))
            .filter(education_completions::Column::EducationCourseId.eq(course.education_course_id))
            .filter(education_completions::Column::CompletedAt.lte(now))
            .order_by_desc(education_completions::Column::CompletedAt)
            .one(db)
            .await
            .map_err(|_| internal_error())?;
        let (status, fulfilled, completion_id, expires_at) = match latest {
            Some(completion)
                if completion_is_current(completion.completed_at, completion.expires_at, now) =>
            {
                (
                    "CURRENT".to_string(),
                    true,
                    Some(completion.education_completion_id),
                    completion.expires_at.map(|ts| ts.to_rfc3339()),
                )
            }
            Some(completion) => (
                "EXPIRED".to_string(),
                false,
                Some(completion.education_completion_id),
                completion.expires_at.map(|ts| ts.to_rfc3339()),
            ),
            None => ("MISSING".to_string(), false, None, None),
        };
        requirements.push(EducationRequirementStatus {
            education_course_id: course.education_course_id,
            course_code: course.course_code,
            title: course.title,
            status,
            fulfilled,
            education_completion_id: completion_id,
            expires_at,
        });
    }

    let fulfilled = requirements.iter().all(|item| item.fulfilled);
    Ok(EducationReadinessResponse {
        employee_id,
        work_id,
        fulfilled,
        requirements,
    })
}
