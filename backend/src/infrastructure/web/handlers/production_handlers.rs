use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::application::dto::CreateProductionDto;
use crate::application::use_cases::ProductionError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{
    bad_request, forbidden, internal_error, FORBIDDEN_EXPLOITATION,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;

pub async fn submit_production(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    dto: web::Json<CreateProductionDto>,
) -> HttpResponse {
    let exploitation_id = path.into_inner();
    if !user.has_exploitation_access(exploitation_id) {
        return forbidden(FORBIDDEN_EXPLOITATION);
    }

    match state
        .production_use_cases
        .submit_production(exploitation_id, dto.into_inner())
        .await
    {
        Ok(production) => HttpResponse::Created().json(production),
        Err(ProductionError::InvalidMonth(msg)) => bad_request(&msg),
        Err(err) => {
            tracing::error!(?err, "submit_production failed");
            internal_error()
        }
    }
}
