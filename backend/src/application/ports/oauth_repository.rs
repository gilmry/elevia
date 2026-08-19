use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::ports::RepoError;
use crate::domain::entities::{AuthorizationCode, OAuthClient, RefreshToken};

pub struct NewOAuthClient {
    pub client_id: String,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
}

#[async_trait]
pub trait OAuthClientRepository: Send + Sync {
    async fn create(&self, new: NewOAuthClient) -> Result<OAuthClient, RepoError>;
    async fn find_by_id(&self, client_id: &str) -> Result<Option<OAuthClient>, RepoError>;
}

pub struct NewAuthorizationCode {
    pub code: String,
    pub client_id: String,
    pub user_id: Uuid,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub expires_at: DateTime<Utc>,
}

#[async_trait]
pub trait AuthorizationCodeRepository: Send + Sync {
    async fn create(&self, new: NewAuthorizationCode) -> Result<(), RepoError>;

    /// Atomically marks the code used and returns its prior row, or `None` if
    /// it doesn't exist or was already used. Atomicity here (a single
    /// `UPDATE ... WHERE used = false RETURNING *`) is what makes a code
    /// single-use even under a concurrent replay attempt.
    async fn consume(&self, code: &str) -> Result<Option<AuthorizationCode>, RepoError>;
}

pub struct NewRefreshToken {
    pub token_hash: String,
    pub client_id: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn create(&self, new: NewRefreshToken) -> Result<(), RepoError>;
    async fn find_valid(&self, token_hash: &str) -> Result<Option<RefreshToken>, RepoError>;
    async fn revoke(&self, token_hash: &str) -> Result<(), RepoError>;
}
