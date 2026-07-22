use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use vespera::axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::models::cargo_items::{self, Entity as CargoItems};
use crate::models::hs_codes::{self, Entity as HsCodes};
use crate::models::un_numbers::{self, Entity as UnNumbers};
use crate::utils::{
    AppState,
    auth::{AdminUser, AuthUser},
};

#[derive(Serialize, vespera::Schema)]
pub struct CargoItemResponse {
    pub cargo_item_id: i64,
    pub bl_number: Option<String>,
    pub dgd_number: Option<String>,
    pub un_number: Option<String>,
    pub dg_class_id: Option<i64>,
    pub hs_code: Option<String>,
    pub item_name: Option<String>,
    pub is_dangerous: bool,
    /// DGD 부재 + 위험물 의심 HS Code → 경고 (FR-B3)
    pub dgd_missing_warning: bool,
    pub arrival_date: Option<String>,
}

impl From<cargo_items::Model> for CargoItemResponse {
    fn from(m: cargo_items::Model) -> Self {
        Self {
            cargo_item_id: m.cargo_item_id,
            bl_number: m.bl_number,
            dgd_number: m.dgd_number,
            un_number: m.un_number,
            dg_class_id: m.dg_class_id,
            hs_code: m.hs_code,
            item_name: m.item_name,
            is_dangerous: m.is_dangerous,
            dgd_missing_warning: m.dgd_missing_warning,
            arrival_date: m.arrival_date.map(|d| d.to_string()),
        }
    }
}

#[derive(Deserialize, vespera::Schema)]
pub struct CreateCargoItemRequest {
    pub cargo_document_id: Option<i64>,
    pub bl_number: Option<String>,
    pub dgd_number: Option<String>,
    pub un_number: Option<String>,
    pub hs_code: Option<String>,
    pub item_name: Option<String>,
    pub description: Option<String>,
    /// YYYY-MM-DD
    pub arrival_date: Option<String>,
}

/// 위험물 등급 판정 결과
struct Classification {
    dg_class_id: Option<i64>,
    is_dangerous: bool,
    dgd_missing_warning: bool,
}

/// UN No./HS Code로 위험물 등급을 확정하고 DGD 누락 여부를 판정 (FR-B1~B3)
async fn classify(
    db: &sea_orm::DatabaseConnection,
    un_number: Option<&str>,
    hs_code: Option<&str>,
    has_dgd: bool,
) -> Result<Classification, StatusCode> {
    // 1순위: UN No. 조회로 Class 확정 (FR-B1)
    if let Some(un) = un_number.filter(|s| !s.is_empty())
        && let Some(row) = UnNumbers::find()
            .filter(un_numbers::Column::UnNumber.eq(un))
            .one(db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(Classification {
            dg_class_id: Some(row.dg_class_id),
            is_dangerous: true,
            dgd_missing_warning: false,
        });
    }

    // 2순위: B/L HS Code 스캔 (FR-B2/B3)
    if let Some(hs) = hs_code.filter(|s| !s.is_empty())
        && let Some(row) = HsCodes::find()
            .filter(hs_codes::Column::HsCode.eq(hs))
            .one(db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        // 위험물 의심 HS Code인데 DGD가 없으면 경고 (FR-B3 — DGD 누락 검증)
        let dgd_missing_warning = row.is_dangerous_suspect && !has_dgd;
        let is_dangerous = row.default_dg_class_id.is_some() || row.is_dangerous_suspect;
        return Ok(Classification {
            dg_class_id: row.default_dg_class_id,
            is_dangerous,
            dgd_missing_warning,
        });
    }

    // 매핑 정보 없음 → 일반 화물 처리
    Ok(Classification {
        dg_class_id: None,
        is_dangerous: false,
        dgd_missing_warning: false,
    })
}

#[derive(Deserialize, vespera::Schema)]
pub struct CargoListQuery {
    /// 입고 예정일 필터 (YYYY-MM-DD)
    pub arrival_date: Option<String>,
}

/// 입고 화물 목록 (arrival_date로 필터 가능)
#[vespera::route(get, tags = ["cargo_items"])]
pub async fn list_cargo_items(
    _auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<CargoListQuery>,
) -> Result<Json<Vec<CargoItemResponse>>, StatusCode> {
    let mut query = CargoItems::find();
    if let Some(date_str) = q.arrival_date.filter(|s| !s.is_empty()) {
        let date = date_str
            .parse::<chrono::NaiveDate>()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        query = query.filter(cargo_items::Column::ArrivalDate.eq(date));
    }
    let rows = query
        .order_by_desc(cargo_items::Column::CargoItemId)
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        rows.into_iter().map(CargoItemResponse::from).collect(),
    ))
}

/// 입고 화물 단건 조회
#[vespera::route(get, path = "/{id}", tags = ["cargo_items"])]
pub async fn get_cargo_item(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<CargoItemResponse>, StatusCode> {
    let row = CargoItems::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(CargoItemResponse::from(row)))
}

/// 입고 화물 등록 (관리 권한): UN No./HS Code로 위험물 등급 자동 확정 + DGD 누락 검증
#[vespera::route(post, tags = ["cargo_items"])]
pub async fn create_cargo_item(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateCargoItemRequest>,
) -> Result<(StatusCode, Json<CargoItemResponse>), StatusCode> {
    let arrival_date = match req.arrival_date.as_deref().filter(|s| !s.is_empty()) {
        Some(s) => Some(
            s.parse::<chrono::NaiveDate>()
                .map_err(|_| StatusCode::BAD_REQUEST)?,
        ),
        None => None,
    };

    let has_dgd = req.dgd_number.as_deref().is_some_and(|s| !s.is_empty());
    let classification = classify(
        &state.db,
        req.un_number.as_deref(),
        req.hs_code.as_deref(),
        has_dgd,
    )
    .await?;

    let new_item = cargo_items::ActiveModel {
        cargo_document_id: Set(req.cargo_document_id),
        bl_number: Set(req.bl_number),
        dgd_number: Set(req.dgd_number),
        un_number: Set(req.un_number),
        dg_class_id: Set(classification.dg_class_id),
        hs_code: Set(req.hs_code),
        item_name: Set(req.item_name),
        description: Set(req.description),
        is_dangerous: Set(classification.is_dangerous),
        dgd_missing_warning: Set(classification.dgd_missing_warning),
        arrival_date: Set(arrival_date),
        ..Default::default()
    };
    let saved = new_item
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(CargoItemResponse::from(saved))))
}
