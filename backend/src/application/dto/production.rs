use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::dto::month::format_month;
use crate::domain::entities::Production;

#[derive(Debug, Deserialize)]
pub struct CreateProductionDto {
    /// "YYYY-MM"
    pub mois: String,
    pub nom: String,
    pub quantite_produite: Decimal,
    pub quantite_vendue: Option<Decimal>,
    pub unite: String,
    pub prix_unitaire_vente: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct ProductionResponseDto {
    pub id: Uuid,
    pub exploitation_id: Uuid,
    pub mois: String,
    pub nom: String,
    pub quantite_produite: Decimal,
    pub quantite_vendue: Option<Decimal>,
    pub unite: String,
    pub prix_unitaire_vente: Option<Decimal>,
}

impl From<Production> for ProductionResponseDto {
    fn from(production: Production) -> Self {
        Self {
            id: production.id,
            exploitation_id: production.exploitation_id,
            mois: format_month(production.mois),
            nom: production.nom,
            quantite_produite: production.quantite_produite,
            quantite_vendue: production.quantite_vendue,
            unite: production.unite,
            prix_unitaire_vente: production.prix_unitaire_vente,
        }
    }
}
