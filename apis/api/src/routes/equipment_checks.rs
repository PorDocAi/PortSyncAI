use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use vespera::axum::{Json, extract::State, http::StatusCode};

use crate::models::equipment::{self, Entity as Equipment};
use crate::models::equipment_check_logs::{self, Entity as EquipmentCheckLogs};
use crate::routes::attendances::{find_today_attendance, today};
use crate::utils::{AppState, auth::AuthUser};

#[derive(Deserialize, vespera::Schema)]
pub struct TagRequest {
    /// 장비에 부착된 NFC 태그 UID (FR-D2)
    pub nfc_tag_uid: String,
}

#[derive(Serialize, vespera::Schema)]
pub struct TagResponse {
    pub equipment_id: i64,
    pub asset_number: Option<String>,
    /// 오늘 이 출근 건으로 태깅 완료한 장비 수
    pub tagged_count_today: u64,
}

/// 장비 NFC 태깅 (FR-D3/D4)
/// 409 = 오늘 이미 다른 출근 건에서 사용된 장비 (돌려쓰기 차단 — 데모 시나리오 2)
#[vespera::route(post, tags = ["equipment_checks"])]
pub async fn tag_equipment(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<TagRequest>,
) -> Result<(StatusCode, Json<TagResponse>), StatusCode> {
    let attendance = find_today_attendance(&state.db, claims.sub)
        .await?
        .ok_or(StatusCode::NOT_FOUND)?;

    let device = Equipment::find()
        .filter(equipment::Column::NfcTagUid.eq(&req.nfc_tag_uid))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if !device.is_active {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    // 돌려쓰기 차단: (장비, 작업일) 기준으로 다른 작업자가 이미 태깅했으면 거부한다.
    // 모바일 NFC 콜백이나 네트워크 재시도로 같은 요청이 반복될 수 있으므로,
    // 동일 작업자의 동일 출근 건 재태깅은 성공 응답을 반환한다.
    let already_used = EquipmentCheckLogs::find()
        .filter(equipment_check_logs::Column::EquipmentId.eq(device.equipment_id))
        .filter(equipment_check_logs::Column::WorkDate.eq(today()))
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if let Some(existing) = already_used {
        if existing.employee_id != claims.sub || existing.attendance_id != attendance.attendance_id
        {
            return Err(StatusCode::CONFLICT);
        }

        let tagged_count_today = EquipmentCheckLogs::find()
            .filter(equipment_check_logs::Column::AttendanceId.eq(attendance.attendance_id))
            .all(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .len() as u64;

        return Ok((
            StatusCode::OK,
            Json(TagResponse {
                equipment_id: device.equipment_id,
                asset_number: device.asset_number,
                tagged_count_today,
            }),
        ));
    }

    let new_log = equipment_check_logs::ActiveModel {
        attendance_id: Set(attendance.attendance_id),
        employee_id: Set(claims.sub),
        equipment_id: Set(device.equipment_id),
        work_date: Set(today()),
        ..Default::default()
    };
    // 동시 요청이 유니크 제약에 걸리는 경우도 409로 처리 (DB가 최종 방어선)
    new_log
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::CONFLICT)?;

    let tagged_count_today = EquipmentCheckLogs::find()
        .filter(equipment_check_logs::Column::AttendanceId.eq(attendance.attendance_id))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len() as u64;

    Ok((
        StatusCode::CREATED,
        Json(TagResponse {
            equipment_id: device.equipment_id,
            asset_number: device.asset_number,
            tagged_count_today,
        }),
    ))
}
