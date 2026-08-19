use actix_web::{web, HttpResponse};
use rust_xlsxwriter::{Chart, ChartType, Format, Workbook, XlsxError};
use uuid::Uuid;

use crate::application::dto::ExportSummaryDto;
use crate::application::use_cases::ExportError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{
    forbidden, internal_error, not_found, FORBIDDEN_EXPLOITATION,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;

const COSTS_SHEET: &str = "Coûts";
const PRODUCTION_SHEET: &str = "Production";

pub async fn export_monthly_xlsx(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let exploitation_id = path.into_inner();
    if !user.has_exploitation_access(exploitation_id) {
        return forbidden(FORBIDDEN_EXPLOITATION);
    }

    let summary = match state
        .export_use_cases
        .monthly_summary(exploitation_id)
        .await
    {
        Ok(summary) => summary,
        Err(ExportError::NotFound) => return not_found("exploitation not found"),
        Err(err) => {
            tracing::error!(?err, "export_monthly_xlsx failed");
            return internal_error();
        }
    };

    let bytes = match build_workbook(&summary) {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(?err, "failed to build xlsx workbook");
            return internal_error();
        }
    };

    HttpResponse::Ok()
        .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .insert_header((
            "Content-Disposition",
            format!(
                "attachment; filename=\"{}-couts-production.xlsx\"",
                sanitize_filename(&summary.exploitation_nom)
            ),
        ))
        .body(bytes)
}

fn sanitize_filename(nom: &str) -> String {
    nom.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect()
}

/// Three sheets: "Coûts" (one row per cost entry - raw, matching what the
/// exploitation actually submits, several rows can share a month),
/// "Production" (one row per month - a single monthly declaration), and
/// "Tableau de bord" (native Excel charts, not images - they stay editable
/// once the file is opened). Costs and production are kept on separate
/// sheets rather than forced onto one shared "row per month": they're
/// recorded at different granularities and mixing them under one header row
/// reads as incoherent.
fn build_workbook(summary: &ExportSummaryDto) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let header_format = Format::new().set_bold();

    let costs_sheet = workbook.add_worksheet();
    costs_sheet.set_name(COSTS_SHEET)?;
    costs_sheet.write_with_format(0, 0, "Mois", &header_format)?;
    costs_sheet.write_with_format(0, 1, "Produit", &header_format)?;
    costs_sheet.write_with_format(0, 2, "Quantité", &header_format)?;
    costs_sheet.write_with_format(0, 3, "Coût", &header_format)?;
    for (i, row) in summary.cost_rows.iter().enumerate() {
        let row_num = (i + 1) as u32;
        costs_sheet.write(row_num, 0, row.mois.as_str())?;
        costs_sheet.write(row_num, 1, row.product_nom.as_str())?;
        costs_sheet.write(row_num, 2, row.quantite)?;
        costs_sheet.write(row_num, 3, row.cout)?;
    }
    costs_sheet.autofit();

    let production_sheet = workbook.add_worksheet();
    production_sheet.set_name(PRODUCTION_SHEET)?;
    let prod_cols = [
        "Mois",
        "Ce qui a été produit",
        "Quantité produite",
        "Quantité vendue",
        "Unité",
        "Prix de vente unitaire",
        "Coût total du mois",
        "Marge estimée",
        "Coût / unité produite",
    ];
    for (col, header) in prod_cols.iter().enumerate() {
        production_sheet.write_with_format(0, col as u16, *header, &header_format)?;
    }
    for (i, row) in summary.production_rows.iter().enumerate() {
        let row_num = (i + 1) as u32;
        production_sheet.write(row_num, 0, row.mois.as_str())?;
        if let Some(nom) = &row.nom {
            production_sheet.write(row_num, 1, nom.as_str())?;
        }
        if let Some(q) = row.quantite_produite {
            production_sheet.write(row_num, 2, q)?;
        }
        if let Some(q) = row.quantite_vendue {
            production_sheet.write(row_num, 3, q)?;
        }
        if let Some(u) = &row.unite {
            production_sheet.write(row_num, 4, u.as_str())?;
        }
        if let Some(p) = row.prix_unitaire_vente {
            production_sheet.write(row_num, 5, p)?;
        }
        production_sheet.write(row_num, 6, row.total_cost)?;
        if let Some(m) = row.estimated_margin {
            production_sheet.write(row_num, 7, m)?;
        }
        if let Some(c) = row.cost_per_unit {
            production_sheet.write(row_num, 8, c)?;
        }
    }
    production_sheet.autofit();

    let last_production_row = summary.production_rows.len() as u32;
    if last_production_row > 0 || !summary.totals_by_product.is_empty() {
        build_dashboard(&mut workbook, summary, last_production_row, &header_format)?;
    }

    workbook.save_to_buffer()
}

fn build_dashboard(
    workbook: &mut Workbook,
    summary: &ExportSummaryDto,
    last_production_row: u32,
    header_format: &Format,
) -> Result<(), XlsxError> {
    const MOIS_COL: u16 = 0;
    const TOTAL_COST_COL: u16 = 6;
    const MARGIN_COL: u16 = 7;
    const COST_PER_UNIT_COL: u16 = 8;

    let dashboard = workbook.add_worksheet();
    dashboard.set_name("Tableau de bord")?;

    // Small totals-per-intrant table, written here so the pie chart below has
    // something contiguous to point at (the costs sheet is one row per
    // entry, not per intrant - there's no single column to sum).
    let has_products = !summary.totals_by_product.is_empty();
    if has_products {
        dashboard.write_with_format(0, 0, "Intrant", header_format)?;
        dashboard.write_with_format(0, 1, "Coût total période", header_format)?;
        for (i, (nom, total)) in summary.totals_by_product.iter().enumerate() {
            let row = (i + 1) as u32;
            dashboard.write(row, 0, nom.as_str())?;
            dashboard.write(row, 1, *total)?;
        }
        dashboard.autofit();
    }

    let mut chart_row = 0u32;

    if last_production_row > 0 {
        let mut cost_chart = Chart::new(ChartType::Line);
        cost_chart
            .add_series()
            .set_categories((PRODUCTION_SHEET, 1, MOIS_COL, last_production_row, MOIS_COL))
            .set_values((
                PRODUCTION_SHEET,
                1,
                TOTAL_COST_COL,
                last_production_row,
                TOTAL_COST_COL,
            ))
            .set_name("Coût total");
        cost_chart
            .title()
            .set_name("Évolution du coût total par mois");
        cost_chart.legend().set_hidden();
        dashboard.insert_chart(chart_row, 3, &cost_chart)?;
        chart_row += 16;

        let mut margin_chart = Chart::new(ChartType::Column);
        margin_chart
            .add_series()
            .set_categories((PRODUCTION_SHEET, 1, MOIS_COL, last_production_row, MOIS_COL))
            .set_values((
                PRODUCTION_SHEET,
                1,
                MARGIN_COL,
                last_production_row,
                MARGIN_COL,
            ))
            .set_name("Marge estimée");
        margin_chart.title().set_name("Marge estimée par mois");
        margin_chart.legend().set_hidden();
        dashboard.insert_chart(chart_row, 3, &margin_chart)?;
        chart_row += 16;

        let mut cost_per_unit_chart = Chart::new(ChartType::Line);
        cost_per_unit_chart
            .add_series()
            .set_categories((PRODUCTION_SHEET, 1, MOIS_COL, last_production_row, MOIS_COL))
            .set_values((
                PRODUCTION_SHEET,
                1,
                COST_PER_UNIT_COL,
                last_production_row,
                COST_PER_UNIT_COL,
            ))
            .set_name("Coût / unité produite");
        cost_per_unit_chart
            .title()
            .set_name("Coût par unité produite (efficacité)");
        cost_per_unit_chart.legend().set_hidden();
        dashboard.insert_chart(chart_row, 3, &cost_per_unit_chart)?;
        chart_row += 16;
    }

    if has_products {
        let last_product_row = summary.totals_by_product.len() as u32;
        let mut pie_chart = Chart::new(ChartType::Pie);
        pie_chart
            .add_series()
            .set_categories(("Tableau de bord", 1, 0, last_product_row, 0))
            .set_values(("Tableau de bord", 1, 1, last_product_row, 1))
            .set_name("Répartition des coûts par intrant");
        pie_chart
            .title()
            .set_name("Répartition des coûts par intrant (période)");
        dashboard.insert_chart(chart_row, 3, &pie_chart)?;
    }

    Ok(())
}
