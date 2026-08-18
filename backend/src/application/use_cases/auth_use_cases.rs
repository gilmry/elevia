use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::{ChangePasswordRequest, Claims, LoginRequest, LoginResponse};
use crate::application::ports::{RepoError, UserRepository};
use crate::domain::entities::User;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("new password must be at least {MIN_PASSWORD_LENGTH} characters")]
    WeakPassword,
    #[error("password hashing failed")]
    HashingFailed,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

const TOKEN_LIFETIME_HOURS: i64 = 12;
const MIN_PASSWORD_LENGTH: usize = 8;

pub struct AuthUseCases {
    user_repo: Arc<dyn UserRepository>,
    jwt_secret: String,
}

impl AuthUseCases {
    pub fn new(user_repo: Arc<dyn UserRepository>, jwt_secret: String) -> Self {
        Self {
            user_repo,
            jwt_secret,
        }
    }

    pub async fn login(&self, dto: LoginRequest) -> Result<LoginResponse, AuthError> {
        let user = self
            .user_repo
            .find_by_email(&dto.email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let valid = bcrypt::verify(&dto.password, &user.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        let token = self.mint_token(&user, Duration::hours(TOKEN_LIFETIME_HOURS))?;
        Ok(LoginResponse { token })
    }

    /// Signs a JWT for `user` valid for `ttl`. Shared by the direct
    /// email/password login (12h) and the OAuth token endpoint (1h access
    /// tokens, minted from a use case that never sees a password) - both
    /// issue exactly the same claim shape, so `verify_token` and every REST
    /// handler treat the two paths identically.
    pub fn mint_token(&self, user: &User, ttl: Duration) -> Result<String, AuthError> {
        let role = match user.role {
            crate::domain::entities::Role::Admin => "admin",
            crate::domain::entities::Role::Exploitation => "exploitation",
        };

        let exp = (Utc::now() + ttl).timestamp() as usize;
        let claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            role: role.to_string(),
            exploitation_id: user.exploitation_id,
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AuthError::InvalidToken)
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        dto: ChangePasswordRequest,
    ) -> Result<(), AuthError> {
        if dto.new_password.chars().count() < MIN_PASSWORD_LENGTH {
            return Err(AuthError::WeakPassword);
        }

        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let valid = bcrypt::verify(&dto.current_password, &user.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        let password_hash = bcrypt::hash(&dto.new_password, bcrypt::DEFAULT_COST)
            .map_err(|_| AuthError::HashingFailed)?;

        self.user_repo
            .update_password_hash(user_id, password_hash)
            .await?;

        Ok(())
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AuthError::InvalidToken)
    }
}
