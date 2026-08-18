use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use elevia_api::application::use_cases::{
    AdminUseCases, AuthUseCases, CatalogUseCases, CoopUseCases, DashboardUseCases, EntryUseCases,
    OAuthUseCases, ProductionUseCases,
};
use elevia_api::infrastructure::database::create_pool;
use elevia_api::infrastructure::database::repositories::{
    PostgresAuthorizationCodeRepository, PostgresEntryRepository, PostgresExploitationRepository,
    PostgresOAuthClientRepository, PostgresProductRepository, PostgresProductionRepository,
    PostgresRefreshTokenRepository, PostgresUserRepository,
};
use elevia_api::infrastructure::web::{configure_routes, AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid port number");

    let db = create_pool(&database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run database migrations");

    elevia_api::infrastructure::bootstrap::bootstrap_admin(&db).await;

    let exploitation_repo = Arc::new(PostgresExploitationRepository::new(db.clone()));
    let user_repo = Arc::new(PostgresUserRepository::new(db.clone()));
    let product_repo = Arc::new(PostgresProductRepository::new(db.clone()));
    let entry_repo = Arc::new(PostgresEntryRepository::new(db.clone()));
    let production_repo = Arc::new(PostgresProductionRepository::new(db.clone()));
    let oauth_client_repo = Arc::new(PostgresOAuthClientRepository::new(db.clone()));
    let oauth_code_repo = Arc::new(PostgresAuthorizationCodeRepository::new(db.clone()));
    let oauth_refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(db.clone()));

    let auth_use_cases = Arc::new(AuthUseCases::new(user_repo.clone(), jwt_secret));

    let state = web::Data::new(AppState {
        auth_use_cases: auth_use_cases.clone(),
        entry_use_cases: Arc::new(EntryUseCases::new(entry_repo.clone(), product_repo.clone())),
        production_use_cases: Arc::new(ProductionUseCases::new(production_repo.clone())),
        dashboard_use_cases: Arc::new(DashboardUseCases::new(
            entry_repo.clone(),
            production_repo.clone(),
        )),
        admin_use_cases: Arc::new(AdminUseCases::new(
            exploitation_repo,
            user_repo.clone(),
            product_repo.clone(),
            entry_repo.clone(),
            production_repo.clone(),
        )),
        coop_use_cases: Arc::new(CoopUseCases::new(
            entry_repo,
            production_repo,
            product_repo.clone(),
        )),
        catalog_use_cases: Arc::new(CatalogUseCases::new(product_repo)),
        oauth_use_cases: Arc::new(OAuthUseCases::new(
            oauth_client_repo,
            oauth_code_repo,
            oauth_refresh_repo,
            user_repo,
            auth_use_cases,
        )),
    });

    tracing::info!("elevia-api listening on {host}:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Cors::permissive())
            .wrap(tracing_actix_web::TracingLogger::default())
            .configure(configure_routes)
    })
    .bind((host, port))?
    .run()
    .await
}
