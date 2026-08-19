use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::application::ports::{EntryRepository, NewEntry, RepoError};
use crate::domain::entities::Entry;
use crate::infrastructure::database::DbPool;

pub struct PostgresEntryRepository {
    pool: DbPool,
}

impl PostgresEntryRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EntryRepository for PostgresEntryRepository {
    async fn upsert(&self, new: NewEntry) -> Result<Entry, RepoError> {
        sqlx::query_as::<_, Entry>(
            "INSERT INTO entries (exploitation_id, product_id, mois, quantite, cout)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (exploitation_id, product_id, mois)
             DO UPDATE SET quantite = EXCLUDED.quantite, cout = EXCLUDED.cout
             RETURNING id, exploitation_id, product_id, mois, quantite, cout, created_at",
        )
        .bind(new.exploitation_id)
        .bind(new.product_id)
        .bind(new.mois)
        .bind(new.quantite)
        .bind(new.cout)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn list_by_exploitation(&self, exploitation_id: Uuid) -> Result<Vec<Entry>, RepoError> {
        sqlx::query_as::<_, Entry>(
            "SELECT id, exploitation_id, product_id, mois, quantite, cout, created_at
             FROM entries WHERE exploitation_id = $1 ORDER BY mois, product_id",
        )
        .bind(exploitation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn delete(&self, id: Uuid, exploitation_id: Uuid) -> Result<bool, RepoError> {
        let result = sqlx::query("DELETE FROM entries WHERE id = $1 AND exploitation_id = $2")
            .bind(id)
            .bind(exploitation_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn monthly_cost_totals(
        &self,
        exploitation_id: Uuid,
    ) -> Result<Vec<(NaiveDate, Decimal)>, RepoError> {
        sqlx::query_as::<_, (NaiveDate, Decimal)>(
            "SELECT mois, COALESCE(SUM(cout), 0) FROM entries
             WHERE exploitation_id = $1 GROUP BY mois ORDER BY mois",
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
            "SELECT EXISTS (SELECT 1 FROM entries WHERE exploitation_id = $1 AND mois = $2)",
        )
        .bind(exploitation_id)
        .bind(mois)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn total_cost_by_exploitation_for_month(
        &self,
        mois: NaiveDate,
    ) -> Result<Vec<(Uuid, Decimal)>, RepoError> {
        sqlx::query_as::<_, (Uuid, Decimal)>(
            "SELECT exploitation_id, COALESCE(SUM(cout), 0) FROM entries
             WHERE mois = $1 GROUP BY exploitation_id",
        )
        .bind(mois)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn sum_quantity_by_product_for_month(
        &self,
        mois: NaiveDate,
    ) -> Result<Vec<(Uuid, Decimal)>, RepoError> {
        sqlx::query_as::<_, (Uuid, Decimal)>(
            "SELECT product_id, COALESCE(SUM(quantite), 0) FROM entries
             WHERE mois = $1 GROUP BY product_id",
        )
        .bind(mois)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
}
