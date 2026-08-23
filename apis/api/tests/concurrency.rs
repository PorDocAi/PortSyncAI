//! 장비 NFC 태깅 동시성 테스트 (AC-7 — #42)
//!
//! AC-7: 같은 장비를 10명이 동시에 태깅하면 정확히 1건만 201을 받고
//! 나머지 9건은 409로 거부된다. 최종 방어선은 DB의 (장비, 작업일)
//! 유니크 제약이므로, 요청 인터리빙 순서와 무관하게 결과 집합이 결정적이다.
//! 하네스스 방침과 동일하게 sleep·폴링은 일절 없다 — JoinSet으로 요청을
//! 한꺼번에 기동하고 응답 상태로만 단언한다.

mod common;

use std::sync::Arc;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::task::JoinSet;
use vespera::axum::http::StatusCode;

/// AC-7 — 동시 태깅 10개 요청 (같은 장비) → 정확히 1건 201, 나머지 9건 409
#[tokio::test]
async fn ten_concurrent_tags_yield_one_created_and_nine_conflicts() {
    let app = Arc::new(common::spawn_app().await);
    let device = common::seed_active_equipment(&app.db).await;

    // 픽스처: 작업원 10명 (출근 절차 완료 상태). 생성은 직렬로 하고,
    // 동시성은 태깅 요청에만 부여한다 — 시나리오의 대상은 태깅 경합이다.
    let mut participants = Vec::with_capacity(10);
    for i in 0..10 {
        let (worker, _) = common::spawn_worker(&app.db, &format!("nfc-ac7-{i:02}")).await;
        participants.push((
            worker.employee_id,
            common::bearer_for(worker.employee_id, "WORKER"),
        ));
    }

    let device_uid = device.nfc_tag_uid.clone();
    let mut set = JoinSet::new();
    for (employee_id, bearer) in participants {
        let app = Arc::clone(&app);
        let uid = device_uid.clone();
        set.spawn(async move {
            let response = app.tag_equipment_with(&bearer, &uid).await;
            (employee_id, response)
        });
    }

    let mut results = Vec::with_capacity(10);
    while let Some(joined) = set.join_next().await {
        results.push(joined.expect("동시 태깅 태스크 패닉"));
    }

    // 정확히 하나가 이기고, 나머지는 전부 409다 (다른 상태 코드는 허용하지 않는다)
    assert_eq!(results.len(), 10, "모든 요청에 대한 응답을 받아야 한다");
    let mut responses = results.into_iter();
    let (winner_id, winner_response) = responses
        .find(|(_, response)| response.status() == StatusCode::CREATED)
        .expect("동시 요청 중 정확히 하나는 201이어야 한다");
    assert!(
        responses.all(|(_, response)| response.status() == StatusCode::CONFLICT),
        "승자를 제외한 나머지 9건은 모두 409여야 한다"
    );

    // 승자 응답도 정상 본문을 가진다 — 오늘자 태깅 수는 자기 자신 1건
    assert_eq!(winner_response.json()["tagged_count_today"], 1);

    // 로그는 첫 태깅 1건만 남고, 소유자는 201을 받은 작업원이다
    let logs = api::models::equipment_check_logs::Entity::find()
        .filter(api::models::equipment_check_logs::Column::EquipmentId.eq(device.equipment_id))
        .all(&app.db)
        .await
        .expect("태깅 로그 조회 실패");
    assert_eq!(logs.len(), 1, "동시 태깅이 정리되면 로그는 1건뿐이다");
    assert_eq!(logs[0].employee_id, winner_id);
}
