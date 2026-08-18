use crate::application::ports::RepoError;
use crate::domain::entities::Entry;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct NewEntry {
    pub exploitation_id: Uuid,
    pub product_id: Uuid,
    pub mois: NaiveDate,
    pub quantite: Decimal,
    pub cout: Decimal,
}

#[async_trait]
pub trait EntryRepository: Send + Sync {
    /// Creates the entry, or replaces it if one already exists for the same
    /// exploitation/product/month (resubmitting a month's costs is expected).
    async fn upsert(&self, new: NewEntry) -> Result<Entry, RepoError>;

    async fn list_by_exploitation(&self, exploitation_id: Uuid) -> Result<Vec<Entry>, RepoError>;

    /// Total cost per month for one exploitation, oldest first.
    async fn monthly_cost_totals(
        &self,
        exploitation_id: Uuid,
    ) -> Result<Vec<(NaiveDate, Decimal)>, RepoError>;

    async fn exists_for_month(
        &self,
        exploitation_id: Uuid,
        mois: NaiveDate,
    ) -> Result<bool, RepoError>;

    /// Total cost per exploitation for a given month (coop-wide comparison).
    async fn total_cost_by_exploitation_for_month(
        &self,
        mois: NaiveDate,
    ) -> Result<Vec<(Uuid, Decimal)>, RepoError>;

    /// Total quantity entered per product for a given month, across all exploitations
    /// (used to size grouped input purchases).
    async fn sum_quantity_by_product_for_month(
        &self,
        mois: NaiveDate,
    ) -> Result<Vec<(Uuid, Decimal)>, RepoError>;
}
