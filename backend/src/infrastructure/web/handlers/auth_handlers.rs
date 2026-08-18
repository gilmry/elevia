use actix_web::{web, HttpResponse};

use crate::application::dto::LoginRequest;
use crate::application::use_cases::AuthError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{internal_error, unauthorized};

pub async fn login(state: web::Data<AppState>, dto: web::Json<LoginRequest>) -> HttpResponse {
    match state.auth_use_cases.login(dto.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(AuthError::InvalidCredentials) => unauthorized("invalid email or password"),
        Err(err) => {
            tracing::error!(?err, "login failed");
            internal_error()
        }
    }
}
