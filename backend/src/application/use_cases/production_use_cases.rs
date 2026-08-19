use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::month::parse_month;
use crate::application::dto::{CreateProductionDto, ProductionResponseDto};
use crate::application::ports::{NewProduction, ProductionRepository, RepoError};

#[derive(Debug, Error)]
pub enum ProductionError {
    #[error("invalid month: {0}")]
    InvalidMonth(String),
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

pub struct ProductionUseCases {
    production_repo: Arc<dyn ProductionRepository>,
}

impl ProductionUseCases {
    pub fn new(production_repo: Arc<dyn ProductionRepository>) -> Self {
        Self { production_repo }
    }

    pub async fn submit_production(
        &self,
        exploitation_id: Uuid,
        dto: CreateProductionDto,
    ) -> Result<ProductionResponseDto, ProductionError> {
        let mois = parse_month(&dto.mois).map_err(ProductionError::InvalidMonth)?;

        let production = self
            .production_repo
            .upsert(NewProduction {
                exploitation_id,
                mois,
                nom: dto.nom,
                quantite_produite: dto.quantite_produite,
                quantite_vendue: dto.quantite_vendue,
                unite: dto.unite,
                prix_unitaire_vente: dto.prix_unitaire_vente,
            })
            .await?;

        Ok(production.into())
    }
}
