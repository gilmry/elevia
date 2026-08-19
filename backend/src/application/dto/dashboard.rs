use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

/// Stats for one thing produced in one month. A farm can produce several
/// distinct things the same month (eggs and meat, say) - `cost_per_unit` and
/// `estimated_margin` are both computed against that month's *shared* total
/// cost, since costs aren't allocated per product; this is a simplification,
/// not an accounting split.
#[derive(Debug, Serialize)]
pub struct ProductionStatsDto {
    pub nom: String,
    pub quantite_produite: Decimal,
    pub quantite_vendue: Option<Decimal>,
    pub unite: String,
    pub prix_unitaire_vente: Option<Decimal>,
    pub cost_per_unit: Option<Decimal>,
    pub estimated_margin: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct MonthlyStatsDto {
    /// "YYYY-MM"
    pub mois: String,
    pub total_cost: Decimal,
    /// Empty if no production was declared that month.
    pub productions: Vec<ProductionStatsDto>,
}

#[derive(Debug, Serialize)]
pub struct ExploitationDashboardDto {
    pub exploitation_id: Uuid,
    /// Oldest month first.
    pub monthly: Vec<MonthlyStatsDto>,
    /// Total cost per intrant over the whole history - for the "répartition
    /// des coûts par intrant" chart, which needs a period total rather than
    /// a monthly breakdown.
    pub totals_by_product: Vec<(String, Decimal)>,
}
