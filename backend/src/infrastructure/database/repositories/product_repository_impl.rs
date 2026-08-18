use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::{NewProduct, ProductChanges, ProductRepository, RepoError};
use crate::domain::entities::Product;
use crate::infrastructure::database::DbPool;

pub struct PostgresProductRepository {
    pool: DbPool,
}

impl PostgresProductRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PostgresProductRepository {
    async fn create(&self, new: NewProduct) -> Result<Product, RepoError> {
        sqlx::query_as::<_, Product>(
            "INSERT INTO products (nom, unite, categorie) VALUES ($1, $2, $3)
             RETURNING id, nom, unite, categorie",
        )
        .bind(new.nom)
        .bind(new.unite)
        .bind(new.categorie)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn update(&self, id: Uuid, changes: ProductChanges) -> Result<Product, RepoError> {
        sqlx::query_as::<_, Product>(
            "UPDATE products
             SET nom = COALESCE($2, nom),
                 unite = COALESCE($3, unite),
                 categorie = COALESCE($4, categorie)
             WHERE id = $1
             RETURNING id, nom, unite, categorie",
        )
        .bind(id)
        .bind(changes.nom)
        .bind(changes.unite)
        .bind(changes.categorie)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, RepoError> {
        sqlx::query_as::<_, Product>("SELECT id, nom, unite, categorie FROM products WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn list_all(&self) -> Result<Vec<Product>, RepoError> {
        sqlx::query_as::<_, Product>(
            "SELECT id, nom, unite, categorie FROM products ORDER BY categorie, nom",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
}
