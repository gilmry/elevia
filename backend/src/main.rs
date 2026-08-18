use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use elevia_api::infrastructure::database::create_pool;
use elevia_api::infrastructure::web::{configure_routes, AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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

    tracing::info!("elevia-api listening on {host}:{port}");

    let state = web::Data::new(AppState { db });

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
