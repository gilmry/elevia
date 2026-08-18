use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::month::parse_month;
use crate::application::dto::{CreateEntryDto, EntryResponseDto};
use crate::application::ports::{EntryRepository, NewEntry, ProductRepository, RepoError};

#[derive(Debug, Error)]
pub enum EntryError {
    #[error("invalid month: {0}")]
    InvalidMonth(String),
    #[error("unknown product")]
    UnknownProduct,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

pub struct EntryUseCases {
    entry_repo: Arc<dyn EntryRepository>,
    product_repo: Arc<dyn ProductRepository>,
}

impl EntryUseCases {
    pub fn new(
        entry_repo: Arc<dyn EntryRepository>,
        product_repo: Arc<dyn ProductRepository>,
    ) -> Self {
        Self {
            entry_repo,
            product_repo,
        }
    }

    pub async fn submit_entry(
        &self,
        exploitation_id: Uuid,
        dto: CreateEntryDto,
    ) -> Result<EntryResponseDto, EntryError> {
        let mois = parse_month(&dto.mois).map_err(EntryError::InvalidMonth)?;

        if self
            .product_repo
            .find_by_id(dto.product_id)
            .await?
            .is_none()
        {
            return Err(EntryError::UnknownProduct);
        }

        let entry = self
            .entry_repo
            .upsert(NewEntry {
                exploitation_id,
                product_id: dto.product_id,
                mois,
                quantite: dto.quantite,
                cout: dto.cout,
            })
            .await?;

        Ok(entry.into())
    }

    pub async fn list_entries(
        &self,
        exploitation_id: Uuid,
    ) -> Result<Vec<EntryResponseDto>, EntryError> {
        let entries = self
            .entry_repo
            .list_by_exploitation(exploitation_id)
            .await?;
        Ok(entries.into_iter().map(Into::into).collect())
    }
}
