use crate::application::ports::RepoError;
use crate::domain::entities::Production;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct NewProduction {
    pub exploitation_id: Uuid,
    pub mois: NaiveDate,
    pub nom: String,
    pub quantite_produite: Decimal,
    pub quantite_vendue: Option<Decimal>,
    pub unite: String,
    pub prix_unitaire_vente: Option<Decimal>,
}

#[async_trait]
pub trait ProductionRepository: Send + Sync {
    /// Creates the production record, or replaces it if one already exists for the
    /// same exploitation/month (resubmitting a month's production is expected).
    async fn upsert(&self, new: NewProduction) -> Result<Production, RepoError>;

    async fn list_by_exploitation(
        &self,
        exploitation_id: Uuid,
    ) -> Result<Vec<Production>, RepoError>;

    async fn exists_for_month(
        &self,
        exploitation_id: Uuid,
        mois: NaiveDate,
    ) -> Result<bool, RepoError>;

    /// Every exploitation's production for a given month (coop-wide comparison).
    async fn all_for_month(&self, mois: NaiveDate) -> Result<Vec<Production>, RepoError>;
}
