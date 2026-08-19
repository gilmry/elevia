use std::collections::HashMap;

use actix_web::{web, HttpResponse};
use rust_xlsxwriter::{Format, Workbook, XlsxError};
use uuid::Uuid;

use crate::application::dto::ExportSummaryDto;
use crate::application::use_cases::ExportError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{
    forbidden, internal_error, not_found, FORBIDDEN_EXPLOITATION,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;

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

/// One sheet, one row per month, one column per intrant actually used - the
/// same shape as the paper/Excel ledger a farmer already keeps, so the file
/// they download is familiar rather than a database dump.
fn build_workbook(summary: &ExportSummaryDto) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Saisie mensuelle")?;

    let header_format = Format::new().set_bold();

    let mut col = 0u16;
    sheet.write_with_format(0, col, "Mois", &header_format)?;
    for product_name in &summary.product_columns {
        col += 1;
        sheet.write_with_format(0, col, product_name.as_str(), &header_format)?;
    }
    col += 1;
    let total_cost_col = col;
    sheet.write_with_format(0, col, "Coût total", &header_format)?;
    col += 1;
    sheet.write_with_format(0, col, "Ce qui a été produit", &header_format)?;
    col += 1;
    sheet.write_with_format(0, col, "Quantité produite", &header_format)?;
    col += 1;
    sheet.write_with_format(0, col, "Quantité vendue", &header_format)?;
    col += 1;
    sheet.write_with_format(0, col, "Unité", &header_format)?;
    col += 1;
    sheet.write_with_format(0, col, "Prix de vente unitaire", &header_format)?;
    col += 1;
    sheet.write_with_format(0, col, "Marge estimée", &header_format)?;

    for (row_idx, row) in summary.rows.iter().enumerate() {
        let row_num = (row_idx + 1) as u32;
        let costs: HashMap<&str, rust_decimal::Decimal> = row
            .costs_by_product
            .iter()
            .map(|(nom, cout)| (nom.as_str(), *cout))
            .collect();

        let mut col = 0u16;
        sheet.write(row_num, col, row.mois.as_str())?;
        for product_name in &summary.product_columns {
            col += 1;
            if let Some(cout) = costs.get(product_name.as_str()) {
                sheet.write(row_num, col, *cout)?;
            }
        }

        sheet.write(row_num, total_cost_col, row.total_cost)?;
        col = total_cost_col + 1;
        if let Some(nom) = &row.production_nom {
            sheet.write(row_num, col, nom.as_str())?;
        }
        col += 1;
        if let Some(q) = row.quantite_produite {
            sheet.write(row_num, col, q)?;
        }
        col += 1;
        if let Some(q) = row.quantite_vendue {
            sheet.write(row_num, col, q)?;
        }
        col += 1;
        if let Some(u) = &row.unite {
            sheet.write(row_num, col, u.as_str())?;
        }
        col += 1;
        if let Some(p) = row.prix_unitaire_vente {
            sheet.write(row_num, col, p)?;
        }
        col += 1;
        if let Some(m) = row.estimated_margin {
            sheet.write(row_num, col, m)?;
        }
    }

    sheet.autofit();

    workbook.save_to_buffer()
}
