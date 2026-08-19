use rust_decimal::Decimal;
use serde::Serialize;

/// One row per cost entry actually submitted - raw, not aggregated, matching
/// exactly what the exploitation entered (product, quantity, total paid).
#[derive(Debug, Serialize)]
pub struct CostRowDto {
    /// "YYYY-MM"
    pub mois: String,
    pub product_nom: String,
    pub quantite: Decimal,
    pub cout: Decimal,
}

/// One row per month - production is a single monthly declaration, unlike
/// costs which can have several entries (one per product) in the same month.
#[derive(Debug, Serialize)]
pub struct MonthlyProductionRowDto {
    /// "YYYY-MM"
    pub mois: String,
    pub nom: Option<String>,
    pub quantite_produite: Option<Decimal>,
    pub quantite_vendue: Option<Decimal>,
    pub unite: Option<String>,
    pub prix_unitaire_vente: Option<Decimal>,
    /// Sum of that month's cost entries - carried here (not left for a
    /// cross-sheet lookup) so margin and cost/unit are self-contained.
    pub total_cost: Decimal,
    pub estimated_margin: Option<Decimal>,
    pub cost_per_unit: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct ExportSummaryDto {
    pub exploitation_nom: String,
    /// Chronological, one row per cost entry.
    pub cost_rows: Vec<CostRowDto>,
    /// Chronological, one row per month.
    pub production_rows: Vec<MonthlyProductionRowDto>,
    /// Total cost per intrant over the whole period - not derivable from a
    /// single column on either sheet above, computed once here for the
    /// dashboard's pie chart.
    pub totals_by_product: Vec<(String, Decimal)>,
}
