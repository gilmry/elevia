use actix_web::{dev::Payload, error::ErrorUnauthorized, web, Error, FromRequest, HttpRequest};
use std::future::{ready, Ready};
use uuid::Uuid;

use crate::infrastructure::web::app_state::AppState;

/// Authenticated user extracted and verified from the JWT `Authorization: Bearer` header.
///
/// This is the *only* place exploitation isolation is decided: every handler that
/// touches one exploitation's data (`/exploitations/{id}/...`) MUST call
/// `has_exploitation_access` with the `{id}` from the URL before reading or writing
/// anything, regardless of what the client sent. Trusting the URL alone would let any
/// authenticated exploitation read another exploitation's data by editing the path.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
    pub exploitation_id: Option<Uuid>,
}

impl AuthenticatedUser {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// Whether this user may access the given exploitation's data. Admins may access
    /// any exploitation; an exploitation account may only access its own.
    pub fn has_exploitation_access(&self, resource_exploitation_id: Uuid) -> bool {
        if self.is_admin() {
            return true;
        }
        self.exploitation_id == Some(resource_exploitation_id)
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(state) => state,
            None => return ready(Err(ErrorUnauthorized("internal server error"))),
        };

        let auth_header = match req.headers().get("Authorization") {
            Some(header) => match header.to_str() {
                Ok(s) => s,
                Err(_) => return ready(Err(ErrorUnauthorized("invalid authorization header"))),
            },
            None => return ready(Err(ErrorUnauthorized("missing authorization header"))),
        };

        let token = auth_header.trim_start_matches("Bearer ").trim();

        match app_state.auth_use_cases.verify_token(token) {
            Ok(claims) => ready(Ok(AuthenticatedUser {
                user_id: claims.sub,
                email: claims.email,
                role: claims.role,
                exploitation_id: claims.exploitation_id,
            })),
            Err(_) => ready(Err(ErrorUnauthorized("invalid or expired token"))),
        }
    }
}
