use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::{NewUser, RepoError, UserRepository};
use crate::domain::entities::User;
use crate::infrastructure::database::DbPool;

pub struct PostgresUserRepository {
    pool: DbPool,
}

impl PostgresUserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, new: NewUser) -> Result<User, RepoError> {
        sqlx::query_as::<_, User>(
            "INSERT INTO utilisateurs (exploitation_id, email, password_hash, role)
             VALUES ($1, $2, $3, $4)
             RETURNING id, exploitation_id, email, password_hash, role, created_at",
        )
        .bind(new.exploitation_id)
        .bind(new.email)
        .bind(new.password_hash)
        .bind(new.role)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError> {
        sqlx::query_as::<_, User>(
            "SELECT id, exploitation_id, email, password_hash, role, created_at
             FROM utilisateurs WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepoError> {
        sqlx::query_as::<_, User>(
            "SELECT id, exploitation_id, email, password_hash, role, created_at
             FROM utilisateurs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_exploitation_id(
        &self,
        exploitation_id: Uuid,
    ) -> Result<Option<User>, RepoError> {
        // No UNIQUE constraint on exploitation_id - app logic keeps it 1:1, but
        // ORDER BY + LIMIT 1 keeps this deterministic (and error-free) even if
        // that were ever violated, instead of `fetch_optional` erroring on >1 row.
        sqlx::query_as::<_, User>(
            "SELECT id, exploitation_id, email, password_hash, role, created_at
             FROM utilisateurs WHERE exploitation_id = $1
             ORDER BY created_at LIMIT 1",
        )
        .bind(exploitation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn update_password_hash(
        &self,
        user_id: Uuid,
        password_hash: String,
    ) -> Result<(), RepoError> {
        sqlx::query("UPDATE utilisateurs SET password_hash = $1 WHERE id = $2")
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}
