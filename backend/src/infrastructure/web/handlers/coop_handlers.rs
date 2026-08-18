use actix_web::{web, HttpResponse};

use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::internal_error;
use crate::infrastructure::web::middleware::AuthenticatedUser;

/// Shared, aggregated-only view: any authenticated user (exploitation or admin) may
/// read it, since it never exposes another exploitation's nominal figures.
pub async fn coop_dashboard(state: web::Data<AppState>, _user: AuthenticatedUser) -> HttpResponse {
    match state.coop_use_cases.coop_dashboard().await {
        Ok(dashboard) => HttpResponse::Ok().json(dashboard),
        Err(err) => {
            tracing::error!(?err, "coop_dashboard failed");
            internal_error()
        }
    }
}
