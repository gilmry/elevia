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
    /// What was produced (e.g. "Œufs", "Viande de poulet") - free text, not
    /// tied to the intrant catalog, which only covers cost inputs.
    pub nom: String,
    pub quantite_produite: Decimal,
    /// How much of this month's - or an earlier month's carried-over stock -
    /// was actually sold. Distinct from `quantite_produite`: not everything
    /// produced is sold in the same month (or ever). `None` until known.
    pub quantite_vendue: Option<Decimal>,
    pub unite: String,
    /// Optional selling price per unit, used to estimate margin when provided.
    pub prix_unitaire_vente: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}
