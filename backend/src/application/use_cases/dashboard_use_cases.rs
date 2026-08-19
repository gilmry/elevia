use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::month::format_month;
use crate::application::dto::{ExploitationDashboardDto, MonthlyStatsDto, ProductionStatsDto};
use crate::application::ports::{
    EntryRepository, ProductRepository, ProductionRepository, RepoError,
};
use crate::domain::services::{cost_per_unit, estimated_margin};

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

pub struct DashboardUseCases {
    entry_repo: Arc<dyn EntryRepository>,
    production_repo: Arc<dyn ProductionRepository>,
    product_repo: Arc<dyn ProductRepository>,
}

impl DashboardUseCases {
    pub fn new(
        entry_repo: Arc<dyn EntryRepository>,
        production_repo: Arc<dyn ProductionRepository>,
        product_repo: Arc<dyn ProductRepository>,
    ) -> Self {
        Self {
            entry_repo,
            production_repo,
            product_repo,
        }
    }

    pub async fn exploitation_dashboard(
        &self,
        exploitation_id: Uuid,
    ) -> Result<ExploitationDashboardDto, DashboardError> {
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

        let mut total_cost_by_month: BTreeMap<NaiveDate, Decimal> = BTreeMap::new();
        let mut totals_by_product: Vec<(String, Decimal)> = Vec::new();
        for entry in &entries {
            *total_cost_by_month
                .entry(entry.mois)
                .or_insert(Decimal::ZERO) += entry.cout;

            let nom = product_names
                .get(&entry.product_id)
                .cloned()
                .unwrap_or_else(|| "Produit supprimé".to_string());
            match totals_by_product.iter_mut().find(|(n, _)| *n == nom) {
                Some((_, total)) => *total += entry.cout,
                None => totals_by_product.push((nom, entry.cout)),
            }
        }

        // A farm can produce several distinct things the same month (eggs
        // and meat, say) - group by month rather than assuming at most one.
        let mut productions_by_month: BTreeMap<NaiveDate, Vec<_>> = BTreeMap::new();
        for production in productions {
            productions_by_month
                .entry(production.mois)
                .or_default()
                .push(production);
        }

        let mut all_months: Vec<NaiveDate> = total_cost_by_month.keys().copied().collect();
        for mois in productions_by_month.keys() {
            if !all_months.contains(mois) {
                all_months.push(*mois);
            }
        }
        all_months.sort();

        let monthly = all_months
            .into_iter()
            .map(|mois| {
                let total_cost = total_cost_by_month
                    .get(&mois)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let productions = productions_by_month
                    .get(&mois)
                    .map(|ps| {
                        ps.iter()
                            .map(|p| ProductionStatsDto {
                                nom: p.nom.clone(),
                                quantite_produite: p.quantite_produite,
                                quantite_vendue: p.quantite_vendue,
                                unite: p.unite.clone(),
                                prix_unitaire_vente: p.prix_unitaire_vente,
                                cost_per_unit: cost_per_unit(total_cost, p.quantite_produite),
                                estimated_margin: estimated_margin(
                                    total_cost,
                                    p.quantite_vendue,
                                    p.prix_unitaire_vente,
                                ),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                MonthlyStatsDto {
                    mois: format_month(mois),
                    total_cost,
                    productions,
                }
            })
            .collect();

        Ok(ExploitationDashboardDto {
            exploitation_id,
            monthly,
            totals_by_product,
        })
    }
}
