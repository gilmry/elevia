use crate::application::ports::RepoError;
use crate::domain::entities::{Role, User};
use async_trait::async_trait;
use uuid::Uuid;

pub struct NewUser {
    pub exploitation_id: Option<Uuid>,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, new: NewUser) -> Result<User, RepoError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepoError>;
    async fn find_by_exploitation_id(
        &self,
        exploitation_id: Uuid,
    ) -> Result<Option<User>, RepoError>;
    async fn update_password_hash(
        &self,
        user_id: Uuid,
        password_hash: String,
    ) -> Result<(), RepoError>;
}
