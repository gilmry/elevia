use crate::application::ports::RepoError;
use crate::domain::entities::Product;
use async_trait::async_trait;
use uuid::Uuid;

pub struct NewProduct {
    pub nom: String,
    pub unite: String,
    pub categorie: String,
}

#[derive(Default)]
pub struct ProductChanges {
    pub nom: Option<String>,
    pub unite: Option<String>,
    pub categorie: Option<String>,
}

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn create(&self, new: NewProduct) -> Result<Product, RepoError>;
    async fn update(&self, id: Uuid, changes: ProductChanges) -> Result<Product, RepoError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, RepoError>;
    async fn list_all(&self) -> Result<Vec<Product>, RepoError>;
}
