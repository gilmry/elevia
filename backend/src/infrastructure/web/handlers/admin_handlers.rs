use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::application::dto::{CreateExploitationDto, CreateProductDto, UpdateProductDto};
use crate::application::use_cases::AdminError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{
    bad_request, forbidden, internal_error, FORBIDDEN_ADMIN,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;

fn require_admin(user: &AuthenticatedUser) -> Result<(), HttpResponse> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(forbidden(FORBIDDEN_ADMIN))
    }
}

pub async fn create_exploitation(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    dto: web::Json<CreateExploitationDto>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&user) {
        return resp;
    }

    match state
        .admin_use_cases
        .create_exploitation(dto.into_inner())
        .await
    {
        Ok(exploitation) => HttpResponse::Created().json(exploitation),
        Err(AdminError::EmailTaken) => bad_request("an account with this email already exists"),
        Err(err) => {
            tracing::error!(?err, "create_exploitation failed");
            internal_error()
        }
    }
}

pub async fn list_exploitations(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
) -> HttpResponse {
    if let Err(resp) = require_admin(&user) {
        return resp;
    }

    match state.admin_use_cases.list_exploitations_with_status().await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => {
            tracing::error!(?err, "list_exploitations failed");
            internal_error()
        }
    }
}

pub async fn create_product(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    dto: web::Json<CreateProductDto>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&user) {
        return resp;
    }

    match state.admin_use_cases.create_product(dto.into_inner()).await {
        Ok(product) => HttpResponse::Created().json(product),
        Err(err) => {
            tracing::error!(?err, "create_product failed");
            internal_error()
        }
    }
}

pub async fn update_product(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    dto: web::Json<UpdateProductDto>,
) -> HttpResponse {
    if let Err(resp) = require_admin(&user) {
        return resp;
    }

    match state
        .admin_use_cases
        .update_product(path.into_inner(), dto.into_inner())
        .await
    {
        Ok(product) => HttpResponse::Ok().json(product),
        Err(AdminError::UnknownProduct) => bad_request("unknown product"),
        Err(err) => {
            tracing::error!(?err, "update_product failed");
            internal_error()
        }
    }
}
