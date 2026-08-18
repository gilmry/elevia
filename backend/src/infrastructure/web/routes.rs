use crate::infrastructure::web::{handlers, mcp};
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(handlers::health))
        .route("/mcp", web::post().to(mcp::handle))
        .route("/auth/login", web::post().to(handlers::login))
        .route(
            "/auth/change-password",
            web::post().to(handlers::change_password),
        )
        .route(
            "/exploitations/{id}/entries",
            web::post().to(handlers::submit_entry),
        )
        .route(
            "/exploitations/{id}/entries",
            web::get().to(handlers::list_entries),
        )
        .route(
            "/exploitations/{id}/production",
            web::post().to(handlers::submit_production),
        )
        .route(
            "/exploitations/{id}/dashboard",
            web::get().to(handlers::exploitation_dashboard),
        )
        .route(
            "/admin/exploitations",
            web::post().to(handlers::create_exploitation),
        )
        .route(
            "/admin/exploitations",
            web::get().to(handlers::list_exploitations),
        )
        .route(
            "/admin/exploitations/{id}/reset-password",
            web::post().to(handlers::reset_exploitation_password),
        )
        .route("/admin/products", web::post().to(handlers::create_product))
        .route(
            "/admin/products/{id}",
            web::put().to(handlers::update_product),
        )
        .route("/coop/dashboard", web::get().to(handlers::coop_dashboard))
        .route("/products", web::get().to(handlers::list_products));
}
