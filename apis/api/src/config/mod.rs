use sea_orm::{Database, DatabaseConnection};
use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    #[allow(dead_code)]
    pub jwt_secret: String,
    pub upload_dir: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        // 별도 인프라 없이 로컬 개발이 가능하도록 SQLite를 기본값으로 사용한다.
        // 운영 환경에서는 DATABASE_URL을 주입해 PostgreSQL에 연결한다.
        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./test.db?mode=rwc".to_string());
        println!("database_url: {}", database_url);

        Self {
            database_url,
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            // 업로드 경로는 배포 환경에서 영속 볼륨으로 교체할 수 있도록 환경변수로 분리한다.
            upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
        }
    }
}

pub async fn create_db_connection(database_url: &str) -> DatabaseConnection {
    Database::connect(database_url)
        .await
        .expect("Failed to connect to database")
}
