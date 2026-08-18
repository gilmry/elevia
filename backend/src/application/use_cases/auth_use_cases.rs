use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use thiserror::Error;

use crate::application::dto::{Claims, LoginRequest, LoginResponse};
use crate::application::ports::{RepoError, UserRepository};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

const TOKEN_LIFETIME_HOURS: i64 = 12;

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

        let role = match user.role {
            crate::domain::entities::Role::Admin => "admin",
            crate::domain::entities::Role::Exploitation => "exploitation",
        };

        let exp = (Utc::now() + Duration::hours(TOKEN_LIFETIME_HOURS)).timestamp() as usize;
        let claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            role: role.to_string(),
            exploitation_id: user.exploitation_id,
            exp,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(LoginResponse { token })
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
