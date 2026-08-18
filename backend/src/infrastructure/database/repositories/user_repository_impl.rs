use async_trait::async_trait;

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
}
