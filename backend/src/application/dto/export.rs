use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MonthlyExportRowDto {
    /// "YYYY-MM"
    pub mois: String,
    /// Cost per product for that month - only products with an entry that
    /// month are present, looked up against `ExportSummaryDto::product_columns`
    /// when laying out the sheet.
    pub costs_by_product: Vec<(String, Decimal)>,
    pub total_cost: Decimal,
    pub production_nom: Option<String>,
    pub quantite_produite: Option<Decimal>,
    pub quantite_vendue: Option<Decimal>,
    pub unite: Option<String>,
    pub prix_unitaire_vente: Option<Decimal>,
    pub estimated_margin: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct ExportSummaryDto {
    pub exploitation_nom: String,
    /// Ordered union of every product name used across all months - a stable
    /// column set so every row lines up under the same headers, in first-seen
    /// order rather than alphabetical (keeps related intrants grouped the way
    /// the exploitation actually entered them).
    pub product_columns: Vec<String>,
    /// Oldest month first.
    pub rows: Vec<MonthlyExportRowDto>,
}
