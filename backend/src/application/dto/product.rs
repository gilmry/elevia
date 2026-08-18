use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::Product;

#[derive(Debug, Deserialize)]
pub struct CreateProductDto {
    pub nom: String,
    pub unite: String,
    pub categorie: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductDto {
    pub nom: Option<String>,
    pub unite: Option<String>,
    pub categorie: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponseDto {
    pub id: Uuid,
    pub nom: String,
    pub unite: String,
    pub categorie: String,
}

impl From<Product> for ProductResponseDto {
    fn from(product: Product) -> Self {
        Self {
            id: product.id,
            nom: product.nom,
            unite: product.unite,
            categorie: product.categorie,
        }
    }
}
