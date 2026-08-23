pub mod admin;
pub mod attendances;
pub mod work_assignments;
pub mod auth;
pub mod cargo_documents;
pub mod cargo_items;
pub mod departments;
pub mod employees;
pub mod equipment_checks;
pub mod gate;
pub mod job_roles;

use std::collections::HashMap;

use vespera::axum::Json;

#[vespera::route(get, path = "/health")]
pub async fn health() -> Json<HashMap<String, String>> {
    Json(HashMap::from([("status".to_string(), "ok".to_string())]))
}
