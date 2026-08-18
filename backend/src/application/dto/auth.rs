use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

/// JWT claims. `role` is serialized as the plain "admin"/"exploitation" string used
/// by the `user_role` Postgres enum, so it round-trips through `Role` directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// user id
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub exploitation_id: Option<Uuid>,
    /// expiry, seconds since epoch
    pub exp: usize,
}
