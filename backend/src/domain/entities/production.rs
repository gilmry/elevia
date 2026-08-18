use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Production {
    pub id: Uuid,
    pub exploitation_id: Uuid,
    /// First day of the month this production covers.
    pub mois: NaiveDate,
    pub quantite_produite: Decimal,
    pub unite: String,
    /// Optional selling price per unit, used to estimate margin when provided.
    pub prix_unitaire_vente: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}
