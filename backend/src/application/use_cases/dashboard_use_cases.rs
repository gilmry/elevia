use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::month::format_month;
use crate::application::dto::{ExploitationDashboardDto, MonthlyStatsDto};
use crate::application::ports::{EntryRepository, ProductionRepository, RepoError};
use crate::domain::entities::Production;
use crate::domain::services::{cost_per_unit, estimated_margin};

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

pub struct DashboardUseCases {
    entry_repo: Arc<dyn EntryRepository>,
    production_repo: Arc<dyn ProductionRepository>,
}

impl DashboardUseCases {
    pub fn new(
        entry_repo: Arc<dyn EntryRepository>,
        production_repo: Arc<dyn ProductionRepository>,
    ) -> Self {
        Self {
            entry_repo,
            production_repo,
        }
    }

    pub async fn exploitation_dashboard(
        &self,
        exploitation_id: Uuid,
    ) -> Result<ExploitationDashboardDto, DashboardError> {
        let costs = self.entry_repo.monthly_cost_totals(exploitation_id).await?;
        let productions = self
            .production_repo
            .list_by_exploitation(exploitation_id)
            .await?;

        let mut months: BTreeMap<NaiveDate, (Decimal, Option<Production>)> = BTreeMap::new();
        for (mois, total_cost) in costs {
            months.entry(mois).or_insert((Decimal::ZERO, None)).0 = total_cost;
        }
        for production in productions {
            let mois = production.mois;
            months.entry(mois).or_insert((Decimal::ZERO, None)).1 = Some(production);
        }

        let monthly = months
            .into_iter()
            .map(|(mois, (total_cost, production))| {
                let quantity_produced = production.as_ref().map(|p| p.quantite_produite);
                let cost_per_unit_value =
                    quantity_produced.and_then(|q| cost_per_unit(total_cost, q));
                let margin = production.as_ref().and_then(|p| {
                    estimated_margin(total_cost, p.quantite_produite, p.prix_unitaire_vente)
                });

                MonthlyStatsDto {
                    mois: format_month(mois),
                    total_cost,
                    quantity_produced,
                    cost_per_unit: cost_per_unit_value,
                    estimated_margin: margin,
                }
            })
            .collect();

        Ok(ExploitationDashboardDto {
            exploitation_id,
            monthly,
        })
    }
}
