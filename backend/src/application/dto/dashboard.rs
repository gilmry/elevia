use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct MonthlyStatsDto {
    /// "YYYY-MM"
    pub mois: String,
    pub total_cost: Decimal,
    /// What was produced (e.g. "Œufs") - `None` if no production was declared.
    pub nom: Option<String>,
    pub quantity_produced: Option<Decimal>,
    pub quantity_sold: Option<Decimal>,
    pub unite: Option<String>,
    pub cost_per_unit: Option<Decimal>,
    pub estimated_margin: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct ExploitationDashboardDto {
    pub exploitation_id: Uuid,
    /// Oldest month first.
    pub monthly: Vec<MonthlyStatsDto>,
}
