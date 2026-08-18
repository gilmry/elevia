use actix_web::HttpResponse;

/// Liveness/readiness probe used by docker-compose and deployment healthchecks.
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

// auth, entries, production, dashboard, admin and coop handlers land here.
