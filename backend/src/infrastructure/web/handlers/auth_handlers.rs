use actix_web::{web, HttpResponse};

use crate::application::dto::{ChangePasswordRequest, LoginRequest};
use crate::application::use_cases::AuthError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{bad_request, internal_error, unauthorized};
use crate::infrastructure::web::middleware::AuthenticatedUser;

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

pub async fn change_password(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    dto: web::Json<ChangePasswordRequest>,
) -> HttpResponse {
    match state
        .auth_use_cases
        .change_password(user.user_id, dto.into_inner())
        .await
    {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(AuthError::InvalidCredentials) => unauthorized("current password is incorrect"),
        Err(AuthError::WeakPassword) => bad_request("new password must be at least 8 characters"),
        Err(err) => {
            tracing::error!(?err, "change_password failed");
            internal_error()
        }
    }
}
