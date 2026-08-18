use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{
    forbidden, internal_error, FORBIDDEN_EXPLOITATION,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;

pub async fn exploitation_dashboard(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let exploitation_id = path.into_inner();
    if !user.has_exploitation_access(exploitation_id) {
        return forbidden(FORBIDDEN_EXPLOITATION);
    }

    match state
        .dashboard_use_cases
        .exploitation_dashboard(exploitation_id)
        .await
    {
        Ok(dashboard) => HttpResponse::Ok().json(dashboard),
        Err(err) => {
            tracing::error!(?err, "exploitation_dashboard failed");
            internal_error()
        }
    }
}
