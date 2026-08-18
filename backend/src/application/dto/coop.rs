use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ProductNeedDto {
    pub product_id: Uuid,
    pub nom: String,
    pub unite: String,
    pub total_quantite: Decimal,
}

/// Cost-per-unit spread across exploitations, expressed as quartiles rather than
/// per-exploitation figures so no single exploitation's numbers are exposed.
#[derive(Debug, Serialize)]
pub struct QuartilesDto {
    pub q1: Decimal,
    pub median: Decimal,
    pub q3: Decimal,
}

#[derive(Debug, Serialize)]
pub struct CoopDashboardDto {
    /// "YYYY-MM"
    pub mois: String,
    /// Total quantity entered per product this month, across all exploitations -
    /// used to size grouped input purchases.
    pub intrant_needs: Vec<ProductNeedDto>,
    /// Average estimated margin across exploitations that reported one this month.
    pub average_margin: Option<Decimal>,
    pub cost_per_unit_quartiles: Option<QuartilesDto>,
}
