use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::application::ports::{NewProduction, ProductionRepository, RepoError};
use crate::domain::entities::Production;
use crate::infrastructure::database::DbPool;

pub struct PostgresProductionRepository {
    pool: DbPool,
}

impl PostgresProductionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductionRepository for PostgresProductionRepository {
    async fn upsert(&self, new: NewProduction) -> Result<Production, RepoError> {
        sqlx::query_as::<_, Production>(
            "INSERT INTO production (exploitation_id, mois, nom, quantite_produite, quantite_vendue, unite, prix_unitaire_vente)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (exploitation_id, mois)
             DO UPDATE SET nom = EXCLUDED.nom,
                            quantite_produite = EXCLUDED.quantite_produite,
                            quantite_vendue = EXCLUDED.quantite_vendue,
                            unite = EXCLUDED.unite,
                            prix_unitaire_vente = EXCLUDED.prix_unitaire_vente
             RETURNING id, exploitation_id, mois, nom, quantite_produite, quantite_vendue, unite, prix_unitaire_vente, created_at",
        )
        .bind(new.exploitation_id)
        .bind(new.mois)
        .bind(new.nom)
        .bind(new.quantite_produite)
        .bind(new.quantite_vendue)
        .bind(new.unite)
        .bind(new.prix_unitaire_vente)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn list_by_exploitation(
        &self,
        exploitation_id: Uuid,
    ) -> Result<Vec<Production>, RepoError> {
        sqlx::query_as::<_, Production>(
            "SELECT id, exploitation_id, mois, nom, quantite_produite, quantite_vendue, unite, prix_unitaire_vente, created_at
             FROM production WHERE exploitation_id = $1 ORDER BY mois",
        )
        .bind(exploitation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn exists_for_month(
        &self,
        exploitation_id: Uuid,
        mois: NaiveDate,
    ) -> Result<bool, RepoError> {
        let (exists,): (bool,) = sqlx::query_as(
            "SELECT EXISTS (SELECT 1 FROM production WHERE exploitation_id = $1 AND mois = $2)",
        )
        .bind(exploitation_id)
        .bind(mois)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn all_for_month(&self, mois: NaiveDate) -> Result<Vec<Production>, RepoError> {
        sqlx::query_as::<_, Production>(
            "SELECT id, exploitation_id, mois, nom, quantite_produite, quantite_vendue, unite, prix_unitaire_vente, created_at
             FROM production WHERE mois = $1",
        )
        .bind(mois)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
}
