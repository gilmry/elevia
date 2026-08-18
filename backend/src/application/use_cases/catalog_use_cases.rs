use std::sync::Arc;

use thiserror::Error;

use crate::application::dto::ProductResponseDto;
use crate::application::ports::{ProductRepository, RepoError};

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

/// Read-only product catalog, shared by exploitations (to pick a product when
/// submitting costs) and admins alike - unlike `AdminUseCases`, nothing here is
/// privileged.
pub struct CatalogUseCases {
    product_repo: Arc<dyn ProductRepository>,
}

impl CatalogUseCases {
    pub fn new(product_repo: Arc<dyn ProductRepository>) -> Self {
        Self { product_repo }
    }

    pub async fn list_products(&self) -> Result<Vec<ProductResponseDto>, CatalogError> {
        let products = self.product_repo.list_all().await?;
        Ok(products.into_iter().map(Into::into).collect())
    }
}
