use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use vespera::axum::{Json, extract::State, http::StatusCode};

use crate::models::approvals::{self, ApprovalDecision};
use crate::models::attendances::{self, ApprovalStatus, Entity as Attendances};
use crate::models::employees::Entity as Employees;
use crate::utils::{AppState, auth::AdminUser};

#[derive(Serialize, vespera::Schema)]
pub struct PendingApprovalItem {
    pub attendance_id: i64,
    pub employee_id: i64,
    pub employee_name: String,
    pub work_date: String,
    /// 미비 항목 파악용
    pub instruction_ack_completed: bool,
    pub equipment_check_completed: bool,
}

/// 승인 대기 목록 (FR-E1 — 관리자 대기함)
#[vespera::route(get, tags = ["admin"])]
pub async fn list_pending_approvals(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<PendingApprovalItem>>, StatusCode> {
    let rows = Attendances::find()
        .filter(attendances::Column::ApprovalStatus.eq(ApprovalStatus::Pending))
        .find_also_related(Employees)
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items = rows
        .into_iter()
        .map(|(att, emp)| PendingApprovalItem {
            attendance_id: att.attendance_id,
            employee_id: att.employee_id,
            employee_name: emp.map(|e| e.name).unwrap_or_default(),
            work_date: att.work_date.to_string(),
            instruction_ack_completed: att.instruction_ack_completed,
            equipment_check_completed: att.equipment_check_completed,
        })
        .collect();
    Ok(Json(items))
}

#[derive(Deserialize, vespera::Schema)]
pub struct DecideApprovalRequest {
    pub attendance_id: i64,
    /// APPROVED | REJECTED
    pub decision: ApprovalDecision,
    /// 결정 사유 (FR-E2 — 필수)
    pub reason: String,
}

#[derive(Serialize, vespera::Schema)]
pub struct DecideApprovalResponse {
    pub approval_id: i64,
    pub attendance_id: i64,
    pub decision: String,
    pub approval_status: String,
}

/// 승인/반려 결정 (FR-E2): 사유·승인자·시각을 approvals에 기록
#[vespera::route(post, tags = ["admin"])]
pub async fn decide_approval(
    AdminUser(claims): AdminUser,
    State(state): State<AppState>,
    Json(req): Json<DecideApprovalRequest>,
) -> Result<Json<DecideApprovalResponse>, StatusCode> {
    if req.reason.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let attendance = Attendances::find_by_id(req.attendance_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    // PENDING 상태만 결정 가능
    if attendance.approval_status != ApprovalStatus::Pending {
        return Err(StatusCode::CONFLICT);
    }

    let record = approvals::ActiveModel {
        attendance_id: Set(attendance.attendance_id),
        approver_id: Set(claims.sub),
        decision: Set(req.decision.clone()),
        reason: Set(Some(req.reason)),
        ..Default::default()
    };
    let saved = record
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_status = match req.decision {
        ApprovalDecision::Approved => ApprovalStatus::Approved,
        ApprovalDecision::Rejected => ApprovalStatus::Rejected,
    };
    let mut active: attendances::ActiveModel = attendance.into();
    active.approval_status = Set(new_status.clone());
    active.updated_at = Set(Some(chrono::Utc::now().into()));
    active
        .update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DecideApprovalResponse {
        approval_id: saved.approval_id,
        attendance_id: saved.attendance_id,
        decision: match saved.decision {
            ApprovalDecision::Approved => "APPROVED".to_string(),
            ApprovalDecision::Rejected => "REJECTED".to_string(),
        },
        approval_status: crate::routes::attendances::approval_as_str(&new_status).to_string(),
    }))
}
