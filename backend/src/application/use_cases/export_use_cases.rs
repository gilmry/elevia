use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::month::format_month;
use crate::application::dto::{ExportSummaryDto, MonthlyExportRowDto};
use crate::application::ports::{
    EntryRepository, ExploitationRepository, ProductRepository, ProductionRepository, RepoError,
};
use crate::domain::entities::Production;
use crate::domain::services::estimated_margin;

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

    /// One row per month, one column per intrant actually used - the same shape
    /// as the Excel sheet a farmer would keep by hand, so switching to Elevia
    /// doesn't mean losing the format they're used to.
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

        let mut months: BTreeMap<NaiveDate, BTreeMap<String, Decimal>> = BTreeMap::new();
        let mut column_order: Vec<String> = Vec::new();
        for entry in &entries {
            let nom = product_names
                .get(&entry.product_id)
                .cloned()
                .unwrap_or_else(|| "Produit supprimé".to_string());
            if !column_order.contains(&nom) {
                column_order.push(nom.clone());
            }
            months
                .entry(entry.mois)
                .or_default()
                .insert(nom, entry.cout);
        }

        let productions_by_month: HashMap<NaiveDate, Production> =
            productions.into_iter().map(|p| (p.mois, p)).collect();

        // Union of months from both entries and production: a month can have
        // production declared with no costs entered yet, or vice versa.
        let mut all_months: Vec<NaiveDate> = months.keys().copied().collect();
        for mois in productions_by_month.keys() {
            if !all_months.contains(mois) {
                all_months.push(*mois);
            }
        }
        all_months.sort();

        let rows = all_months
            .into_iter()
            .map(|mois| {
                let costs = months.get(&mois).cloned().unwrap_or_default();
                let total_cost: Decimal = costs.values().copied().sum();
                let costs_by_product = costs.into_iter().collect();
                let production = productions_by_month.get(&mois);

                MonthlyExportRowDto {
                    mois: format_month(mois),
                    costs_by_product,
                    total_cost,
                    production_nom: production.map(|p| p.nom.clone()),
                    quantite_produite: production.map(|p| p.quantite_produite),
                    quantite_vendue: production.and_then(|p| p.quantite_vendue),
                    unite: production.map(|p| p.unite.clone()),
                    prix_unitaire_vente: production.and_then(|p| p.prix_unitaire_vente),
                    estimated_margin: production.and_then(|p| {
                        estimated_margin(total_cost, p.quantite_vendue, p.prix_unitaire_vente)
                    }),
                }
            })
            .collect();

        Ok(ExportSummaryDto {
            exploitation_nom: exploitation.nom,
            product_columns: column_order,
            rows,
        })
    }
}
