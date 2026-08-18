use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct MonthlyStatsDto {
    /// "YYYY-MM"
    pub mois: String,
    pub total_cost: Decimal,
    pub quantity_produced: Option<Decimal>,
    pub cost_per_unit: Option<Decimal>,
    pub estimated_margin: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct ExploitationDashboardDto {
    pub exploitation_id: Uuid,
    /// Oldest month first.
    pub monthly: Vec<MonthlyStatsDto>,
}
