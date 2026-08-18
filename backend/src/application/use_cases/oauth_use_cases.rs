use std::sync::Arc;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::{
    AuthorizeFormDto, RegisterClientDto, RegisterClientResponseDto, TokenRequestDto,
    TokenResponseDto,
};
use crate::application::ports::{
    AuthorizationCodeRepository, NewAuthorizationCode, NewOAuthClient, NewRefreshToken,
    OAuthClientRepository, RefreshTokenRepository, RepoError, UserRepository,
};
use crate::application::use_cases::AuthUseCases;
use crate::domain::entities::OAuthClient;

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("redirect_uris must be a non-empty list of absolute http(s) URIs")]
    InvalidRedirectUris,
    #[error("unknown client_id")]
    UnknownClient,
    #[error("redirect_uri is not registered for this client")]
    InvalidRedirectUri,
    #[error("only response_type=code is supported")]
    UnsupportedResponseType,
    #[error("only code_challenge_method=S256 is supported")]
    UnsupportedCodeChallengeMethod,
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("invalid, expired, or already used authorization code")]
    InvalidGrant,
    #[error("PKCE verification failed")]
    InvalidPkce,
    #[error("invalid or expired refresh token")]
    InvalidRefreshToken,
    #[error("unsupported grant_type")]
    UnsupportedGrantType,
    #[error("token signing failed")]
    TokenSigningFailed,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

const AUTH_CODE_TTL_MINUTES: i64 = 10;
const ACCESS_TOKEN_TTL_MINUTES: i64 = 60;
const REFRESH_TOKEN_TTL_DAYS: i64 = 30;

pub struct OAuthUseCases {
    client_repo: Arc<dyn OAuthClientRepository>,
    code_repo: Arc<dyn AuthorizationCodeRepository>,
    refresh_repo: Arc<dyn RefreshTokenRepository>,
    user_repo: Arc<dyn UserRepository>,
    auth_use_cases: Arc<AuthUseCases>,
}

impl OAuthUseCases {
    pub fn new(
        client_repo: Arc<dyn OAuthClientRepository>,
        code_repo: Arc<dyn AuthorizationCodeRepository>,
        refresh_repo: Arc<dyn RefreshTokenRepository>,
        user_repo: Arc<dyn UserRepository>,
        auth_use_cases: Arc<AuthUseCases>,
    ) -> Self {
        Self {
            client_repo,
            code_repo,
            refresh_repo,
            user_repo,
            auth_use_cases,
        }
    }

    /// RFC 7591 dynamic client registration. Public clients only (PKCE, no
    /// secret) - an MCP client like Claude calls this itself on first
    /// connection, there is no manual admin step.
    pub async fn register_client(
        &self,
        dto: RegisterClientDto,
    ) -> Result<RegisterClientResponseDto, OAuthError> {
        if dto.redirect_uris.is_empty()
            || dto
                .redirect_uris
                .iter()
                .any(|uri| !(uri.starts_with("https://") || uri.starts_with("http://localhost")))
        {
            return Err(OAuthError::InvalidRedirectUris);
        }

        let client_id = new_opaque_token();
        let client_name = dto.client_name.unwrap_or_else(|| "MCP client".to_string());

        let client = self
            .client_repo
            .create(NewOAuthClient {
                client_id,
                client_name,
                redirect_uris: dto.redirect_uris,
            })
            .await?;

        Ok(RegisterClientResponseDto {
            client_id: client.client_id,
            client_name: client.client_name,
            redirect_uris: client.redirect_uris,
            token_endpoint_auth_method: "none".to_string(),
            grant_types: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ],
            response_types: vec!["code".to_string()],
        })
    }

    /// Validates `client_id` and `redirect_uri` before anything is rendered
    /// to the browser - this is the one check that must happen before we
    /// trust `redirect_uri` enough to ever send a response there (or an
    /// error) instead of showing a generic in-page error.
    pub async fn validate_authorize_request(
        &self,
        client_id: &str,
        redirect_uri: &str,
        response_type: &str,
        code_challenge_method: &str,
    ) -> Result<OAuthClient, OAuthError> {
        let client = self
            .client_repo
            .find_by_id(client_id)
            .await?
            .ok_or(OAuthError::UnknownClient)?;

        if !client.redirect_uris.iter().any(|uri| uri == redirect_uri) {
            return Err(OAuthError::InvalidRedirectUri);
        }
        if response_type != "code" {
            return Err(OAuthError::UnsupportedResponseType);
        }
        if code_challenge_method != "S256" {
            return Err(OAuthError::UnsupportedCodeChallengeMethod);
        }

        Ok(client)
    }

    /// Verifies the submitted credentials and, on success, mints a one-time
    /// authorization code bound to this client/redirect_uri/PKCE challenge.
    /// Re-validates client_id/redirect_uri (the form fields are
    /// client-supplied, even if hidden) before trusting them again.
    pub async fn authorize(&self, dto: &AuthorizeFormDto) -> Result<String, OAuthError> {
        self.validate_authorize_request(
            &dto.client_id,
            &dto.redirect_uri,
            "code",
            &dto.code_challenge_method,
        )
        .await?;

        let user = self
            .user_repo
            .find_by_email(&dto.email)
            .await?
            .ok_or(OAuthError::InvalidCredentials)?;

        let valid = bcrypt::verify(&dto.password, &user.password_hash)
            .map_err(|_| OAuthError::InvalidCredentials)?;
        if !valid {
            return Err(OAuthError::InvalidCredentials);
        }

        let code = new_opaque_token();
        self.code_repo
            .create(NewAuthorizationCode {
                code: code.clone(),
                client_id: dto.client_id.clone(),
                user_id: user.id,
                redirect_uri: dto.redirect_uri.clone(),
                code_challenge: dto.code_challenge.clone(),
                expires_at: Utc::now() + Duration::minutes(AUTH_CODE_TTL_MINUTES),
            })
            .await?;

        Ok(code)
    }

    pub async fn token(&self, dto: TokenRequestDto) -> Result<TokenResponseDto, OAuthError> {
        match dto.grant_type.as_str() {
            "authorization_code" => self.exchange_code(dto).await,
            "refresh_token" => self.refresh(dto).await,
            _ => Err(OAuthError::UnsupportedGrantType),
        }
    }

    async fn exchange_code(&self, dto: TokenRequestDto) -> Result<TokenResponseDto, OAuthError> {
        let code = dto.code.ok_or(OAuthError::InvalidGrant)?;
        let redirect_uri = dto.redirect_uri.ok_or(OAuthError::InvalidGrant)?;
        let code_verifier = dto.code_verifier.ok_or(OAuthError::InvalidPkce)?;

        let record = self
            .code_repo
            .consume(&code)
            .await?
            .ok_or(OAuthError::InvalidGrant)?;

        if record.expires_at < Utc::now()
            || record.client_id != dto.client_id
            || record.redirect_uri != redirect_uri
        {
            return Err(OAuthError::InvalidGrant);
        }
        if !verify_pkce(&code_verifier, &record.code_challenge) {
            return Err(OAuthError::InvalidPkce);
        }

        let user = self
            .user_repo
            .find_by_id(record.user_id)
            .await?
            .ok_or(OAuthError::InvalidGrant)?;

        self.issue_token_pair(&user, &record.client_id).await
    }

    async fn refresh(&self, dto: TokenRequestDto) -> Result<TokenResponseDto, OAuthError> {
        let presented = dto.refresh_token.ok_or(OAuthError::InvalidRefreshToken)?;
        let token_hash = hash_token(&presented);

        let record = self
            .refresh_repo
            .find_valid(&token_hash)
            .await?
            .ok_or(OAuthError::InvalidRefreshToken)?;

        if record.client_id != dto.client_id {
            return Err(OAuthError::InvalidRefreshToken);
        }

        // Rotate unconditionally: this refresh token is spent whether or not
        // the rest of the exchange succeeds, so a leaked-then-reused token
        // can't be replayed after a legitimate client has already rotated it.
        self.refresh_repo.revoke(&token_hash).await?;

        let user = self
            .user_repo
            .find_by_id(record.user_id)
            .await?
            .ok_or(OAuthError::InvalidRefreshToken)?;

        self.issue_token_pair(&user, &record.client_id).await
    }

    async fn issue_token_pair(
        &self,
        user: &crate::domain::entities::User,
        client_id: &str,
    ) -> Result<TokenResponseDto, OAuthError> {
        let access_token = self
            .auth_use_cases
            .mint_token(user, Duration::minutes(ACCESS_TOKEN_TTL_MINUTES))
            .map_err(|_| OAuthError::TokenSigningFailed)?;

        let refresh_token = new_opaque_token();
        self.refresh_repo
            .create(NewRefreshToken {
                token_hash: hash_token(&refresh_token),
                client_id: client_id.to_string(),
                user_id: user.id,
                expires_at: Utc::now() + Duration::days(REFRESH_TOKEN_TTL_DAYS),
            })
            .await?;

        Ok(TokenResponseDto {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: ACCESS_TOKEN_TTL_MINUTES * 60,
            refresh_token,
        })
    }
}

/// 256 bits of randomness from two v4 UUIDs, hex-concatenated - avoids
/// pulling in a `rand` dependency purely for this; `uuid`'s v4 generator
/// already uses the OS CSPRNG.
fn new_opaque_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn verify_pkce(code_verifier: &str, code_challenge: &str) -> bool {
    let digest = Sha256::digest(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest) == code_challenge
}

fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}
