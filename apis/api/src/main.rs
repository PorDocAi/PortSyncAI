mod config;
// 자동 생성 코드(vespertide export)라 미사용 항목 경고를 모듈 단위로 허용
#[allow(dead_code)]
mod models;
mod routes;
mod utils;

use crate::{
    config::{Config, create_db_connection},
    utils::AppState,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use vespera::axum::extract::DefaultBodyLimit;
use vespera::axum::http::{HeaderValue, Method};

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let db = create_db_connection(&config.database_url).await;
    let state = AppState { db, config };
    let port = state.config.port;
    vespertide::vespertide_migration!(&state.db).await.unwrap();
    utils::seed::load_master_data(&state.db).await;

    let app = vespera::vespera!(
        openapi = ["apps/front/openapi.json", "apps/admin/openapi.json"],
        docs_url = "/docs"
    )
    // Axum multipart 기본 한도(2MiB)를 확장하되, 실제 파일 필드는 20MiB로 더 엄격하게 제한한다.
    // 나머지 1MiB는 multipart 헤더와 일반 폼 필드의 여유분이다.
    .layer(DefaultBodyLimit::max(21 * 1024 * 1024))
    .with_state(state)
    .layer(
        CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                vespera::axum::http::header::CONTENT_TYPE,
                vespera::axum::http::header::AUTHORIZATION,
            ])
            .allow_credentials(true),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("API server is running on port {}", port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    vespera::axum::serve(listener, app).await.unwrap();
}
