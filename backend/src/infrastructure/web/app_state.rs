use std::sync::Arc;

use crate::application::use_cases::{
    AdminUseCases, AuthUseCases, CatalogUseCases, CoopUseCases, DashboardUseCases, EntryUseCases,
    ExportUseCases, OAuthUseCases, ProductionUseCases,
};

#[derive(Clone)]
pub struct AppState {
    pub auth_use_cases: Arc<AuthUseCases>,
    pub entry_use_cases: Arc<EntryUseCases>,
    pub production_use_cases: Arc<ProductionUseCases>,
    pub dashboard_use_cases: Arc<DashboardUseCases>,
    pub admin_use_cases: Arc<AdminUseCases>,
    pub coop_use_cases: Arc<CoopUseCases>,
    pub catalog_use_cases: Arc<CatalogUseCases>,
    pub oauth_use_cases: Arc<OAuthUseCases>,
    pub export_use_cases: Arc<ExportUseCases>,
}
