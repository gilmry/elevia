use std::collections::HashMap;

use actix_web::{web, HttpResponse};
use rust_decimal::Decimal;
use rust_xlsxwriter::{Chart, ChartType, Format, Workbook, XlsxError};
use uuid::Uuid;

use crate::application::dto::ExportSummaryDto;
use crate::application::use_cases::ExportError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{
    forbidden, internal_error, not_found, FORBIDDEN_EXPLOITATION,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;

const DATA_SHEET: &str = "Saisie mensuelle";

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

/// Column layout of the data sheet, computed once so both the data-writing
/// pass and the chart ranges agree on where everything lives.
struct Columns {
    mois: u16,
    products_start: u16,
    total_cost: u16,
    nom: u16,
    quantite_produite: u16,
    quantite_vendue: u16,
    unite: u16,
    prix_unitaire_vente: u16,
    estimated_margin: u16,
    cost_per_unit: u16,
}

impl Columns {
    fn layout(product_count: u16) -> Self {
        let mois = 0;
        let products_start = 1;
        let products_end = products_start + product_count.saturating_sub(1);
        let total_cost = if product_count == 0 {
            products_start
        } else {
            products_end + 1
        };
        Self {
            mois,
            products_start,
            total_cost,
            nom: total_cost + 1,
            quantite_produite: total_cost + 2,
            quantite_vendue: total_cost + 3,
            unite: total_cost + 4,
            prix_unitaire_vente: total_cost + 5,
            estimated_margin: total_cost + 6,
            cost_per_unit: total_cost + 7,
        }
    }
}

/// Two sheets: "Saisie mensuelle" (one row per month, one column per intrant
/// actually used - the same shape as the paper/Excel ledger a farmer already
/// keeps) and "Tableau de bord" (native Excel charts built off that data, not
/// images - they stay editable/interactive once opened).
fn build_workbook(summary: &ExportSummaryDto) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let cols = Columns::layout(summary.product_columns.len() as u16);
    let last_row = summary.rows.len() as u32;

    let header_format = Format::new().set_bold();
    let data_sheet = workbook.add_worksheet();
    data_sheet.set_name(DATA_SHEET)?;

    data_sheet.write_with_format(0, cols.mois, "Mois", &header_format)?;
    for (i, product_name) in summary.product_columns.iter().enumerate() {
        data_sheet.write_with_format(
            0,
            cols.products_start + i as u16,
            product_name.as_str(),
            &header_format,
        )?;
    }
    data_sheet.write_with_format(0, cols.total_cost, "Coût total", &header_format)?;
    data_sheet.write_with_format(0, cols.nom, "Ce qui a été produit", &header_format)?;
    data_sheet.write_with_format(
        0,
        cols.quantite_produite,
        "Quantité produite",
        &header_format,
    )?;
    data_sheet.write_with_format(0, cols.quantite_vendue, "Quantité vendue", &header_format)?;
    data_sheet.write_with_format(0, cols.unite, "Unité", &header_format)?;
    data_sheet.write_with_format(
        0,
        cols.prix_unitaire_vente,
        "Prix de vente unitaire",
        &header_format,
    )?;
    data_sheet.write_with_format(0, cols.estimated_margin, "Marge estimée", &header_format)?;
    data_sheet.write_with_format(
        0,
        cols.cost_per_unit,
        "Coût / unité produite",
        &header_format,
    )?;

    let mut totals_by_product: Vec<(String, Decimal)> = summary
        .product_columns
        .iter()
        .map(|nom| (nom.clone(), Decimal::ZERO))
        .collect();

    for (row_idx, row) in summary.rows.iter().enumerate() {
        let row_num = (row_idx + 1) as u32;
        let costs: HashMap<&str, Decimal> = row
            .costs_by_product
            .iter()
            .map(|(nom, cout)| (nom.as_str(), *cout))
            .collect();

        data_sheet.write(row_num, cols.mois, row.mois.as_str())?;
        for (i, product_name) in summary.product_columns.iter().enumerate() {
            if let Some(cout) = costs.get(product_name.as_str()) {
                data_sheet.write(row_num, cols.products_start + i as u16, *cout)?;
                totals_by_product[i].1 += *cout;
            }
        }

        data_sheet.write(row_num, cols.total_cost, row.total_cost)?;
        if let Some(nom) = &row.production_nom {
            data_sheet.write(row_num, cols.nom, nom.as_str())?;
        }
        if let Some(q) = row.quantite_produite {
            data_sheet.write(row_num, cols.quantite_produite, q)?;
        }
        if let Some(q) = row.quantite_vendue {
            data_sheet.write(row_num, cols.quantite_vendue, q)?;
        }
        if let Some(u) = &row.unite {
            data_sheet.write(row_num, cols.unite, u.as_str())?;
        }
        if let Some(p) = row.prix_unitaire_vente {
            data_sheet.write(row_num, cols.prix_unitaire_vente, p)?;
        }
        if let Some(m) = row.estimated_margin {
            data_sheet.write(row_num, cols.estimated_margin, m)?;
        }
        if let Some(c) = row.cost_per_unit {
            data_sheet.write(row_num, cols.cost_per_unit, c)?;
        }
    }

    data_sheet.autofit();

    if last_row > 0 {
        build_dashboard(&mut workbook, &cols, last_row, &totals_by_product)?;
    }

    workbook.save_to_buffer()
}

fn build_dashboard(
    workbook: &mut Workbook,
    cols: &Columns,
    last_row: u32,
    totals_by_product: &[(String, Decimal)],
) -> Result<(), XlsxError> {
    let header_format = Format::new().set_bold();
    let dashboard = workbook.add_worksheet();
    dashboard.set_name("Tableau de bord")?;

    // Small totals-per-intrant table, written here so the pie chart below has
    // something contiguous to point at (the data sheet's product columns
    // hold per-month values, not the period total each slice needs).
    let has_products = !totals_by_product.is_empty();
    if has_products {
        dashboard.write_with_format(0, 0, "Intrant", &header_format)?;
        dashboard.write_with_format(0, 1, "Coût total période", &header_format)?;
        for (i, (nom, total)) in totals_by_product.iter().enumerate() {
            let row = (i + 1) as u32;
            dashboard.write(row, 0, nom.as_str())?;
            dashboard.write(row, 1, *total)?;
        }
        dashboard.autofit();
    }

    let mut chart_row = 0u32;

    let mut cost_chart = Chart::new(ChartType::Line);
    cost_chart
        .add_series()
        .set_categories((DATA_SHEET, 1, cols.mois, last_row, cols.mois))
        .set_values((DATA_SHEET, 1, cols.total_cost, last_row, cols.total_cost))
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
        .set_categories((DATA_SHEET, 1, cols.mois, last_row, cols.mois))
        .set_values((
            DATA_SHEET,
            1,
            cols.estimated_margin,
            last_row,
            cols.estimated_margin,
        ))
        .set_name("Marge estimée");
    margin_chart.title().set_name("Marge estimée par mois");
    margin_chart.legend().set_hidden();
    dashboard.insert_chart(chart_row, 3, &margin_chart)?;
    chart_row += 16;

    let mut cost_per_unit_chart = Chart::new(ChartType::Line);
    cost_per_unit_chart
        .add_series()
        .set_categories((DATA_SHEET, 1, cols.mois, last_row, cols.mois))
        .set_values((
            DATA_SHEET,
            1,
            cols.cost_per_unit,
            last_row,
            cols.cost_per_unit,
        ))
        .set_name("Coût / unité produite");
    cost_per_unit_chart
        .title()
        .set_name("Coût par unité produite (efficacité)");
    cost_per_unit_chart.legend().set_hidden();
    dashboard.insert_chart(chart_row, 3, &cost_per_unit_chart)?;

    if has_products {
        let last_product_row = totals_by_product.len() as u32;
        let mut pie_chart = Chart::new(ChartType::Pie);
        pie_chart
            .add_series()
            .set_categories(("Tableau de bord", 1, 0, last_product_row, 0))
            .set_values(("Tableau de bord", 1, 1, last_product_row, 1))
            .set_name("Répartition des coûts par intrant");
        pie_chart
            .title()
            .set_name("Répartition des coûts par intrant (période)");
        dashboard.insert_chart(chart_row + 16, 3, &pie_chart)?;
    }

    Ok(())
}
