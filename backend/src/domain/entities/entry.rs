use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Entry {
    pub id: Uuid,
    pub exploitation_id: Uuid,
    pub product_id: Uuid,
    /// First day of the month this entry covers.
    pub mois: NaiveDate,
    pub quantite: Decimal,
    pub cout: Decimal,
    pub created_at: DateTime<Utc>,
}
