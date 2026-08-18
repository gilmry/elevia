use actix_web::{web, HttpResponse};

use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::internal_error;
use crate::infrastructure::web::middleware::AuthenticatedUser;

/// Any authenticated user (exploitation or admin) may list products - an
/// exploitation needs the catalog to pick a product when submitting costs.
pub async fn list_products(state: web::Data<AppState>, _user: AuthenticatedUser) -> HttpResponse {
    match state.catalog_use_cases.list_products().await {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(err) => {
            tracing::error!(?err, "list_products failed");
            internal_error()
        }
    }
}
