use std::collections::HashMap;
use std::sync::Arc;

use rust_decimal::Decimal;
use thiserror::Error;

use crate::application::dto::month::{current_month_start, format_month};
use crate::application::dto::{CoopDashboardDto, ProductNeedDto, QuartilesDto};
use crate::application::ports::{
    EntryRepository, ProductRepository, ProductionRepository, RepoError,
};
use crate::domain::services::{cost_per_unit, estimated_margin};

#[derive(Debug, Error)]
pub enum CoopError {
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

pub struct CoopUseCases {
    entry_repo: Arc<dyn EntryRepository>,
    production_repo: Arc<dyn ProductionRepository>,
    product_repo: Arc<dyn ProductRepository>,
}

impl CoopUseCases {
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

    /// Coop-wide, aggregated-only view for the current month: never exposes a single
    /// exploitation's nominal figures (see `quartiles`), only totals and spread.
    pub async fn coop_dashboard(&self) -> Result<CoopDashboardDto, CoopError> {
        let mois = current_month_start();

        let quantities = self
            .entry_repo
            .sum_quantity_by_product_for_month(mois)
            .await?;
        let mut intrant_needs = Vec::with_capacity(quantities.len());
        for (product_id, total_quantite) in quantities {
            if let Some(product) = self.product_repo.find_by_id(product_id).await? {
                intrant_needs.push(ProductNeedDto {
                    product_id,
                    nom: product.nom,
                    unite: product.unite,
                    total_quantite,
                });
            }
        }

        let cost_by_exploitation: HashMap<_, _> = self
            .entry_repo
            .total_cost_by_exploitation_for_month(mois)
            .await?
            .into_iter()
            .collect();
        let productions = self.production_repo.all_for_month(mois).await?;

        let mut margins = Vec::new();
        let mut costs_per_unit = Vec::new();
        for production in &productions {
            let total_cost = cost_by_exploitation
                .get(&production.exploitation_id)
                .copied()
                .unwrap_or(Decimal::ZERO);

            if let Some(cpu) = cost_per_unit(total_cost, production.quantite_produite) {
                costs_per_unit.push(cpu);
            }
            if let Some(margin) = estimated_margin(
                total_cost,
                production.quantite_vendue,
                production.prix_unitaire_vente,
            ) {
                margins.push(margin);
            }
        }

        Ok(CoopDashboardDto {
            mois: format_month(mois),
            intrant_needs,
            average_margin: average(&margins),
            cost_per_unit_quartiles: quartiles(&mut costs_per_unit),
        })
    }
}

fn average(values: &[Decimal]) -> Option<Decimal> {
    if values.is_empty() {
        return None;
    }
    let sum: Decimal = values.iter().sum();
    Some(sum / Decimal::from(values.len() as u64))
}

fn quartiles(values: &mut [Decimal]) -> Option<QuartilesDto> {
    if values.is_empty() {
        return None;
    }
    values.sort();
    Some(QuartilesDto {
        q1: nearest_rank_percentile(values, 0.25),
        median: nearest_rank_percentile(values, 0.5),
        q3: nearest_rank_percentile(values, 0.75),
    })
}

/// Nearest-rank percentile (no interpolation) - simple and sufficient given the
/// small number of exploitations this coop dashboard aggregates over.
fn nearest_rank_percentile(sorted: &[Decimal], p: f64) -> Decimal {
    let n = sorted.len();
    let rank = ((p * n as f64).ceil() as usize).clamp(1, n);
    sorted[rank - 1]
}
