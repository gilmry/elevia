mod admin_handlers;
mod auth_handlers;
mod coop_handlers;
mod dashboard_handlers;
mod entry_handlers;
mod production_handlers;
mod responses;

use actix_web::HttpResponse;

pub use admin_handlers::{create_exploitation, create_product, list_exploitations, update_product};
pub use auth_handlers::login;
pub use coop_handlers::coop_dashboard;
pub use dashboard_handlers::exploitation_dashboard;
pub use entry_handlers::{list_entries, submit_entry};
pub use production_handlers::submit_production;

/// Liveness/readiness probe used by docker-compose and deployment healthchecks.
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}
