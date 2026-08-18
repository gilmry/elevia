use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::month::current_month_start;
use crate::application::dto::{
    CreateExploitationDto, CreateProductDto, ExploitationResponseDto, ExploitationStatusDto,
    ProductResponseDto, ResetPasswordRequest, UpdateProductDto,
};
use crate::application::ports::{
    EntryRepository, ExploitationRepository, NewExploitation, NewProduct, NewUser, ProductChanges,
    ProductRepository, ProductionRepository, RepoError, UserRepository,
};
use crate::domain::entities::Role;

const MIN_PASSWORD_LENGTH: usize = 8;

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("an account with this email already exists")]
    EmailTaken,
    #[error("unknown product")]
    UnknownProduct,
    #[error("unknown exploitation")]
    UnknownExploitation,
    #[error("new password must be at least {MIN_PASSWORD_LENGTH} characters")]
    WeakPassword,
    #[error("password hashing failed")]
    HashingFailed,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

pub struct AdminUseCases {
    exploitation_repo: Arc<dyn ExploitationRepository>,
    user_repo: Arc<dyn UserRepository>,
    product_repo: Arc<dyn ProductRepository>,
    entry_repo: Arc<dyn EntryRepository>,
    production_repo: Arc<dyn ProductionRepository>,
}

impl AdminUseCases {
    pub fn new(
        exploitation_repo: Arc<dyn ExploitationRepository>,
        user_repo: Arc<dyn UserRepository>,
        product_repo: Arc<dyn ProductRepository>,
        entry_repo: Arc<dyn EntryRepository>,
        production_repo: Arc<dyn ProductionRepository>,
    ) -> Self {
        Self {
            exploitation_repo,
            user_repo,
            product_repo,
            entry_repo,
            production_repo,
        }
    }

    pub async fn create_exploitation(
        &self,
        dto: CreateExploitationDto,
    ) -> Result<ExploitationResponseDto, AdminError> {
        if self.user_repo.find_by_email(&dto.email).await?.is_some() {
            return Err(AdminError::EmailTaken);
        }

        let exploitation = self
            .exploitation_repo
            .create(NewExploitation {
                nom: dto.nom,
                contact: dto.contact,
            })
            .await?;

        let password_hash = bcrypt::hash(&dto.password, bcrypt::DEFAULT_COST)
            .map_err(|_| AdminError::HashingFailed)?;

        self.user_repo
            .create(NewUser {
                exploitation_id: Some(exploitation.id),
                email: dto.email,
                password_hash,
                role: Role::Exploitation,
            })
            .await?;

        Ok(exploitation.into())
    }

    pub async fn list_exploitations_with_status(
        &self,
    ) -> Result<Vec<ExploitationStatusDto>, AdminError> {
        let current_month = current_month_start();
        let exploitations = self.exploitation_repo.list_all().await?;

        let mut statuses = Vec::with_capacity(exploitations.len());
        for exploitation in exploitations {
            let entries_submitted = self
                .entry_repo
                .exists_for_month(exploitation.id, current_month)
                .await?;
            let production_submitted = self
                .production_repo
                .exists_for_month(exploitation.id, current_month)
                .await?;

            statuses.push(ExploitationStatusDto {
                id: exploitation.id,
                nom: exploitation.nom,
                entries_submitted,
                production_submitted,
            });
        }

        Ok(statuses)
    }

    pub async fn create_product(
        &self,
        dto: CreateProductDto,
    ) -> Result<ProductResponseDto, AdminError> {
        let product = self
            .product_repo
            .create(NewProduct {
                nom: dto.nom,
                unite: dto.unite,
                categorie: dto.categorie,
            })
            .await?;
        Ok(product.into())
    }

    pub async fn reset_password(
        &self,
        exploitation_id: Uuid,
        dto: ResetPasswordRequest,
    ) -> Result<(), AdminError> {
        if dto.new_password.len() < MIN_PASSWORD_LENGTH {
            return Err(AdminError::WeakPassword);
        }

        let user = self
            .user_repo
            .find_by_exploitation_id(exploitation_id)
            .await?
            .ok_or(AdminError::UnknownExploitation)?;

        let password_hash = bcrypt::hash(&dto.new_password, bcrypt::DEFAULT_COST)
            .map_err(|_| AdminError::HashingFailed)?;

        self.user_repo
            .update_password_hash(user.id, password_hash)
            .await?;

        Ok(())
    }

    pub async fn update_product(
        &self,
        id: Uuid,
        dto: UpdateProductDto,
    ) -> Result<ProductResponseDto, AdminError> {
        if self.product_repo.find_by_id(id).await?.is_none() {
            return Err(AdminError::UnknownProduct);
        }

        let product = self
            .product_repo
            .update(
                id,
                ProductChanges {
                    nom: dto.nom,
                    unite: dto.unite,
                    categorie: dto.categorie,
                },
            )
            .await?;
        Ok(product.into())
    }
}
