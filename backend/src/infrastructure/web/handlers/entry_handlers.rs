use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::application::dto::CreateEntryDto;
use crate::application::use_cases::EntryError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{
    bad_request, forbidden, internal_error, FORBIDDEN_EXPLOITATION,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;

pub async fn submit_entry(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    dto: web::Json<CreateEntryDto>,
) -> HttpResponse {
    let exploitation_id = path.into_inner();
    if !user.has_exploitation_access(exploitation_id) {
        return forbidden(FORBIDDEN_EXPLOITATION);
    }

    match state
        .entry_use_cases
        .submit_entry(exploitation_id, dto.into_inner())
        .await
    {
        Ok(entry) => HttpResponse::Created().json(entry),
        Err(EntryError::InvalidMonth(msg)) => bad_request(&msg),
        Err(EntryError::UnknownProduct) => bad_request("unknown product"),
        Err(err) => {
            tracing::error!(?err, "submit_entry failed");
            internal_error()
        }
    }
}

pub async fn list_entries(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let exploitation_id = path.into_inner();
    if !user.has_exploitation_access(exploitation_id) {
        return forbidden(FORBIDDEN_EXPLOITATION);
    }

    match state.entry_use_cases.list_entries(exploitation_id).await {
        Ok(entries) => HttpResponse::Ok().json(entries),
        Err(err) => {
            tracing::error!(?err, "list_entries failed");
            internal_error()
        }
    }
}
