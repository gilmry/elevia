use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::dto::month::format_month;
use crate::domain::entities::Entry;

#[derive(Debug, Deserialize)]
pub struct CreateEntryDto {
    pub product_id: Uuid,
    /// "YYYY-MM"
    pub mois: String,
    pub quantite: Decimal,
    pub cout: Decimal,
}

#[derive(Debug, Serialize)]
pub struct EntryResponseDto {
    pub id: Uuid,
    pub exploitation_id: Uuid,
    pub product_id: Uuid,
    pub mois: String,
    pub quantite: Decimal,
    pub cout: Decimal,
}

impl From<Entry> for EntryResponseDto {
    fn from(entry: Entry) -> Self {
        Self {
            id: entry.id,
            exploitation_id: entry.exploitation_id,
            product_id: entry.product_id,
            mois: format_month(entry.mois),
            quantite: entry.quantite,
            cout: entry.cout,
        }
    }
}
