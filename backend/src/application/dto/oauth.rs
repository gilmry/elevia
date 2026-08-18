use serde::{Deserialize, Serialize};

/// RFC 7591 dynamic client registration request. Only public clients (PKCE,
/// no client secret) are supported, so there is nothing to negotiate beyond
/// the redirect URIs the client will use.
#[derive(Debug, Deserialize)]
pub struct RegisterClientDto {
    pub client_name: Option<String>,
    pub redirect_uris: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterClientResponseDto {
    pub client_id: String,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub token_endpoint_auth_method: String,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
}

/// The `/oauth/authorize` form (GET query string, then POST as
/// `application/x-www-form-urlencoded` per the OAuth spec - not JSON).
#[derive(Debug, Deserialize)]
pub struct AuthorizeParams {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

// A flat struct, not `AuthorizeParams` behind `#[serde(flatten)]`: the form
// is deserialized by `serde_urlencoded` (via actix's `Form` extractor),
// which does not support flatten reliably.
#[derive(Debug, Deserialize)]
pub struct AuthorizeFormDto {
    pub email: String,
    pub password: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

/// `/oauth/token` request body - also `application/x-www-form-urlencoded`.
/// One struct covers both grant types since form bodies can't easily be
/// deserialized as an enum; fields irrelevant to the grant in use are
/// simply `None`.
#[derive(Debug, Deserialize)]
pub struct TokenRequestDto {
    pub grant_type: String,
    pub client_id: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub redirect_uri: Option<String>,
    #[serde(default)]
    pub code_verifier: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponseDto {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
}
