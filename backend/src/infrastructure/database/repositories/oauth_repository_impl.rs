use async_trait::async_trait;
use chrono::Utc;

use crate::application::ports::{
    AuthorizationCodeRepository, NewAuthorizationCode, NewOAuthClient, NewRefreshToken,
    OAuthClientRepository, RefreshTokenRepository, RepoError,
};
use crate::domain::entities::{AuthorizationCode, OAuthClient, RefreshToken};
use crate::infrastructure::database::DbPool;

pub struct PostgresOAuthClientRepository {
    pool: DbPool,
}

impl PostgresOAuthClientRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OAuthClientRepository for PostgresOAuthClientRepository {
    async fn create(&self, new: NewOAuthClient) -> Result<OAuthClient, RepoError> {
        sqlx::query_as::<_, OAuthClient>(
            "INSERT INTO oauth_clients (client_id, client_name, redirect_uris)
             VALUES ($1, $2, $3)
             RETURNING client_id, client_name, redirect_uris, created_at",
        )
        .bind(new.client_id)
        .bind(new.client_name)
        .bind(new.redirect_uris)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_id(&self, client_id: &str) -> Result<Option<OAuthClient>, RepoError> {
        sqlx::query_as::<_, OAuthClient>(
            "SELECT client_id, client_name, redirect_uris, created_at
             FROM oauth_clients WHERE client_id = $1",
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
}

pub struct PostgresAuthorizationCodeRepository {
    pool: DbPool,
}

impl PostgresAuthorizationCodeRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorizationCodeRepository for PostgresAuthorizationCodeRepository {
    async fn create(&self, new: NewAuthorizationCode) -> Result<(), RepoError> {
        sqlx::query(
            "INSERT INTO oauth_authorization_codes
                 (code, client_id, user_id, redirect_uri, code_challenge, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(new.code)
        .bind(new.client_id)
        .bind(new.user_id)
        .bind(new.redirect_uri)
        .bind(new.code_challenge)
        .bind(new.expires_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    async fn consume(&self, code: &str) -> Result<Option<AuthorizationCode>, RepoError> {
        sqlx::query_as::<_, AuthorizationCode>(
            "UPDATE oauth_authorization_codes SET used = true
             WHERE code = $1 AND used = false
             RETURNING code, client_id, user_id, redirect_uri, code_challenge, expires_at, used",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
}

pub struct PostgresRefreshTokenRepository {
    pool: DbPool,
}

impl PostgresRefreshTokenRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RefreshTokenRepository for PostgresRefreshTokenRepository {
    async fn create(&self, new: NewRefreshToken) -> Result<(), RepoError> {
        sqlx::query(
            "INSERT INTO oauth_refresh_tokens (token_hash, client_id, user_id, expires_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(new.token_hash)
        .bind(new.client_id)
        .bind(new.user_id)
        .bind(new.expires_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    async fn find_valid(&self, token_hash: &str) -> Result<Option<RefreshToken>, RepoError> {
        sqlx::query_as::<_, RefreshToken>(
            "SELECT token_hash, client_id, user_id, expires_at, revoked
             FROM oauth_refresh_tokens
             WHERE token_hash = $1 AND revoked = false AND expires_at > $2",
        )
        .bind(token_hash)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn revoke(&self, token_hash: &str) -> Result<(), RepoError> {
        sqlx::query("UPDATE oauth_refresh_tokens SET revoked = true WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}
