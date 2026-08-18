use crate::infrastructure::web::handlers;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(handlers::health));

    // /auth/login, /exploitations/{id}/..., /admin/..., /coop/dashboard
    // are wired here once the corresponding use cases exist.
}
