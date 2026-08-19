use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::month::format_month;
use crate::application::dto::{CostRowDto, ExportSummaryDto, MonthlyProductionRowDto};
use crate::application::ports::{
    EntryRepository, ExploitationRepository, ProductRepository, ProductionRepository, RepoError,
};
use crate::domain::entities::Production;
use crate::domain::services::{cost_per_unit, estimated_margin};

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("exploitation not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

pub struct ExportUseCases {
    exploitation_repo: Arc<dyn ExploitationRepository>,
    entry_repo: Arc<dyn EntryRepository>,
    production_repo: Arc<dyn ProductionRepository>,
    product_repo: Arc<dyn ProductRepository>,
}

impl ExportUseCases {
    pub fn new(
        exploitation_repo: Arc<dyn ExploitationRepository>,
        entry_repo: Arc<dyn EntryRepository>,
        production_repo: Arc<dyn ProductionRepository>,
        product_repo: Arc<dyn ProductRepository>,
    ) -> Self {
        Self {
            exploitation_repo,
            entry_repo,
            production_repo,
            product_repo,
        }
    }

    /// Costs and production are kept as two separate row sets - a cost entry
    /// is submitted per product (several per month), production is a single
    /// monthly declaration, so forcing both onto one "row per month" mixes
    /// two different granularities under the same headers.
    pub async fn monthly_summary(
        &self,
        exploitation_id: Uuid,
    ) -> Result<ExportSummaryDto, ExportError> {
        let exploitation = self
            .exploitation_repo
            .find_by_id(exploitation_id)
            .await?
            .ok_or(ExportError::NotFound)?;

        let entries = self
            .entry_repo
            .list_by_exploitation(exploitation_id)
            .await?;
        let productions = self
            .production_repo
            .list_by_exploitation(exploitation_id)
            .await?;
        let products = self.product_repo.list_all().await?;
        let product_names: HashMap<Uuid, String> =
            products.into_iter().map(|p| (p.id, p.nom)).collect();

        let mut cost_rows: Vec<CostRowDto> = Vec::with_capacity(entries.len());
        let mut total_cost_by_month: BTreeMap<NaiveDate, Decimal> = BTreeMap::new();
        let mut totals_by_product: Vec<(String, Decimal)> = Vec::new();

        for entry in &entries {
            let nom = product_names
                .get(&entry.product_id)
                .cloned()
                .unwrap_or_else(|| "Produit supprimé".to_string());

            cost_rows.push(CostRowDto {
                mois: format_month(entry.mois),
                product_nom: nom.clone(),
                quantite: entry.quantite,
                cout: entry.cout,
            });

            *total_cost_by_month
                .entry(entry.mois)
                .or_insert(Decimal::ZERO) += entry.cout;

            match totals_by_product.iter_mut().find(|(n, _)| *n == nom) {
                Some((_, total)) => *total += entry.cout,
                None => totals_by_product.push((nom, entry.cout)),
            }
        }
        cost_rows.sort_by(|a, b| a.mois.cmp(&b.mois).then(a.product_nom.cmp(&b.product_nom)));

        // A farm can produce several distinct things the same month (eggs
        // and meat, say) - group by month rather than assuming at most one,
        // so each becomes its own row instead of silently overwriting.
        let mut productions_by_month: BTreeMap<NaiveDate, Vec<Production>> = BTreeMap::new();
        for production in productions {
            productions_by_month
                .entry(production.mois)
                .or_default()
                .push(production);
        }

        // Union of months from both costs and production: a month can have
        // production declared with no costs entered yet, or vice versa.
        let mut all_months: Vec<NaiveDate> = total_cost_by_month.keys().copied().collect();
        for mois in productions_by_month.keys() {
            if !all_months.contains(mois) {
                all_months.push(*mois);
            }
        }
        all_months.sort();

        let mut production_rows = Vec::new();
        for mois in all_months {
            let total_cost = total_cost_by_month
                .get(&mois)
                .copied()
                .unwrap_or(Decimal::ZERO);
            match productions_by_month.get(&mois) {
                Some(productions) => {
                    for p in productions {
                        production_rows.push(MonthlyProductionRowDto {
                            mois: format_month(mois),
                            nom: Some(p.nom.clone()),
                            quantite_produite: Some(p.quantite_produite),
                            quantite_vendue: p.quantite_vendue,
                            unite: Some(p.unite.clone()),
                            prix_unitaire_vente: p.prix_unitaire_vente,
                            total_cost,
                            estimated_margin: estimated_margin(
                                total_cost,
                                p.quantite_vendue,
                                p.prix_unitaire_vente,
                            ),
                            cost_per_unit: cost_per_unit(total_cost, p.quantite_produite),
                        });
                    }
                }
                // Costs entered but nothing produced yet that month - still
                // worth a row so the month isn't silently missing.
                None => production_rows.push(MonthlyProductionRowDto {
                    mois: format_month(mois),
                    nom: None,
                    quantite_produite: None,
                    quantite_vendue: None,
                    unite: None,
                    prix_unitaire_vente: None,
                    total_cost,
                    estimated_margin: None,
                    cost_per_unit: None,
                }),
            }
        }

        Ok(ExportSummaryDto {
            exploitation_nom: exploitation.nom,
            cost_rows,
            production_rows,
            totals_by_product,
        })
    }
}
