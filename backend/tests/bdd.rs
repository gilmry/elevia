//! BDD isolation test: verifies exploitation A can never read exploitation B's
//! entries or dashboard, even by editing the exploitation id in the URL, and that
//! this is enforced by the backend (not just trusted from the client) - see
//! `tests/features/isolation.feature`.
//!
//! Runs the real Actix server against a throwaway Postgres testcontainer and drives
//! it over HTTP, so it exercises the actual JWT/URL isolation boundary
//! (`AuthenticatedUser::has_exploitation_access` in
//! `src/infrastructure/web/middleware.rs`), not just the use case layer.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use actix_web::{web, App, HttpServer};
use cucumber::{given, then, when, World};
use elevia_api::application::use_cases::{
    AdminUseCases, AuthUseCases, CatalogUseCases, CoopUseCases, DashboardUseCases, EntryUseCases,
    ProductionUseCases,
};
use elevia_api::infrastructure::database::create_pool;
use elevia_api::infrastructure::database::repositories::{
    PostgresEntryRepository, PostgresExploitationRepository, PostgresProductRepository,
    PostgresProductionRepository, PostgresUserRepository,
};
use elevia_api::infrastructure::web::{configure_routes, AppState};
use serde_json::json;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use uuid::Uuid;

const JWT_SECRET: &str = "test-secret-for-bdd-isolation";
const ADMIN_EMAIL: &str = "admin@elevia.test";
const ADMIN_PASSWORD: &str = "adminpass123";
const EXPLOITATION_PASSWORD: &str = "password123";

#[derive(World)]
#[world(init = Self::new)]
pub struct IsolationWorld {
    _container: Option<ContainerAsync<Postgres>>,
    base_url: String,
    client: reqwest::Client,
    tokens: HashMap<String, String>,
    exploitation_ids: HashMap<String, Uuid>,
    product_id: Option<Uuid>,
    current_user: Option<String>,
    last_status: Option<u16>,
}

impl std::fmt::Debug for IsolationWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsolationWorld")
            .field("base_url", &self.base_url)
            .field("current_user", &self.current_user)
            .field("last_status", &self.last_status)
            .finish()
    }
}

impl IsolationWorld {
    async fn new() -> Self {
        Self {
            _container: None,
            base_url: String::new(),
            client: reqwest::Client::new(),
            tokens: HashMap::new(),
            exploitation_ids: HashMap::new(),
            product_id: None,
            current_user: None,
            last_status: None,
        }
    }

    /// Starts a throwaway Postgres container and a real instance of the Actix server
    /// bound to a random port, once per scenario. Idempotent within a scenario since
    /// several Background steps call it.
    async fn ensure_server_running(&mut self) {
        if !self.base_url.is_empty() {
            return;
        }

        let container = Postgres::default()
            .start()
            .await
            .expect("failed to start postgres testcontainer");
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("failed to get postgres port");
        let database_url = format!("postgresql://postgres:postgres@127.0.0.1:{port}/postgres");

        let pool = create_pool(&database_url)
            .await
            .expect("failed to connect to test database");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");

        let exploitation_repo = Arc::new(PostgresExploitationRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let product_repo = Arc::new(PostgresProductRepository::new(pool.clone()));
        let entry_repo = Arc::new(PostgresEntryRepository::new(pool.clone()));
        let production_repo = Arc::new(PostgresProductionRepository::new(pool.clone()));

        let state = web::Data::new(AppState {
            auth_use_cases: Arc::new(AuthUseCases::new(user_repo.clone(), JWT_SECRET.to_string())),
            entry_use_cases: Arc::new(EntryUseCases::new(entry_repo.clone(), product_repo.clone())),
            production_use_cases: Arc::new(ProductionUseCases::new(production_repo.clone())),
            dashboard_use_cases: Arc::new(DashboardUseCases::new(
                entry_repo.clone(),
                production_repo.clone(),
            )),
            admin_use_cases: Arc::new(AdminUseCases::new(
                exploitation_repo,
                user_repo,
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
        });

        let server = HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .configure(configure_routes)
        })
        .bind(("127.0.0.1", 0))
        .expect("failed to bind test server");

        let addr = server.addrs()[0];
        self.base_url = format!("http://{addr}");
        tokio::spawn(server.run());
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Seed the first admin account directly - the API has no public endpoint to
        // create one (an existing admin is required to create exploitations/products).
        let password_hash =
            bcrypt::hash(ADMIN_PASSWORD, bcrypt::DEFAULT_COST).expect("failed to hash password");
        sqlx::query(
            "INSERT INTO utilisateurs (exploitation_id, email, password_hash, role)
             VALUES (NULL, $1, $2, 'admin')",
        )
        .bind(ADMIN_EMAIL)
        .bind(password_hash)
        .execute(&pool)
        .await
        .expect("failed to seed admin user");

        self._container = Some(container);

        let token = self.login(ADMIN_EMAIL, ADMIN_PASSWORD).await;
        self.tokens.insert("admin".to_string(), token);
    }

    async fn login(&self, email: &str, password: &str) -> String {
        let response = self
            .client
            .post(format!("{}/auth/login", self.base_url))
            .json(&json!({ "email": email, "password": password }))
            .send()
            .await
            .expect("login request failed");
        assert!(
            response.status().is_success(),
            "login failed with status {}",
            response.status()
        );
        let body: serde_json::Value = response.json().await.expect("invalid login response");
        body["token"]
            .as_str()
            .expect("login response missing token")
            .to_string()
    }

    fn token_for(&self, user: &str) -> String {
        self.tokens
            .get(user)
            .unwrap_or_else(|| panic!("no token recorded for '{user}'"))
            .clone()
    }

    fn id_for(&self, exploitation: &str) -> Uuid {
        *self
            .exploitation_ids
            .get(exploitation)
            .unwrap_or_else(|| panic!("unknown exploitation '{exploitation}'"))
    }
}

#[given("un compte administrateur existe")]
async fn given_admin_exists(world: &mut IsolationWorld) {
    world.ensure_server_running().await;
}

#[given(regex = r#"^l'exploitation "([^"]+)" existe avec un compte utilisateur$"#)]
async fn given_exploitation_exists(world: &mut IsolationWorld, nom: String) {
    world.ensure_server_running().await;

    let email = format!("{}@elevia.test", nom.to_lowercase().replace(' ', "-"));
    let admin_token = world.token_for("admin");

    let response = world
        .client
        .post(format!("{}/admin/exploitations", world.base_url))
        .bearer_auth(&admin_token)
        .json(&json!({
            "nom": nom,
            "contact": format!("{nom} <contact@example.com>"),
            "email": email,
            "password": EXPLOITATION_PASSWORD,
        }))
        .send()
        .await
        .expect("create exploitation request failed");
    assert!(
        response.status().is_success(),
        "failed to create exploitation '{nom}': {}",
        response.status()
    );

    let body: serde_json::Value = response.json().await.expect("invalid response body");
    let id: Uuid = body["id"]
        .as_str()
        .expect("missing exploitation id")
        .parse()
        .expect("exploitation id is not a valid uuid");
    world.exploitation_ids.insert(nom.clone(), id);

    let token = world.login(&email, EXPLOITATION_PASSWORD).await;
    world.tokens.insert(nom, token);
}

#[given(regex = r#"^"([^"]+)" a soumis des coûts et une production pour le mois courant$"#)]
async fn given_month_submitted(world: &mut IsolationWorld, nom: String) {
    let admin_token = world.token_for("admin");

    if world.product_id.is_none() {
        let response = world
            .client
            .post(format!("{}/admin/products", world.base_url))
            .bearer_auth(&admin_token)
            .json(&json!({ "nom": "Provende", "unite": "kg", "categorie": "intrant" }))
            .send()
            .await
            .expect("create product request failed");
        assert!(response.status().is_success(), "failed to create product");
        let body: serde_json::Value = response.json().await.expect("invalid response body");
        let id: Uuid = body["id"]
            .as_str()
            .expect("missing product id")
            .parse()
            .expect("product id is not a valid uuid");
        world.product_id = Some(id);
    }

    let exploitation_id = world.id_for(&nom);
    let token = world.token_for(&nom);
    let mois = chrono::Utc::now().format("%Y-%m").to_string();

    let response = world
        .client
        .post(format!(
            "{}/exploitations/{exploitation_id}/entries",
            world.base_url
        ))
        .bearer_auth(&token)
        .json(&json!({
            "product_id": world.product_id.unwrap(),
            "mois": mois,
            "quantite": "100",
            "cout": "250.50",
        }))
        .send()
        .await
        .expect("submit entry request failed");
    assert!(response.status().is_success(), "failed to submit entry");

    let response = world
        .client
        .post(format!(
            "{}/exploitations/{exploitation_id}/production",
            world.base_url
        ))
        .bearer_auth(&token)
        .json(&json!({
            "mois": mois,
            "nom": "Blé",
            "quantite_produite": "40",
            "quantite_vendue": "40",
            "unite": "tonnes",
            "prix_unitaire_vente": "180",
        }))
        .send()
        .await
        .expect("submit production request failed");
    assert!(
        response.status().is_success(),
        "failed to submit production"
    );
}

#[given(regex = r#"^je suis authentifié en tant qu'utilisateur de "([^"]+)"$"#)]
async fn given_authenticated_as(world: &mut IsolationWorld, nom: String) {
    world.current_user = Some(nom);
}

#[when(regex = r#"^je demande les entries de "([^"]+)" en modifiant l'identifiant dans l'URL$"#)]
async fn when_request_other_entries(world: &mut IsolationWorld, target: String) {
    let target_id = world.id_for(&target);
    let current = world
        .current_user
        .clone()
        .expect("no authenticated user set");
    let token = world.token_for(&current);

    let response = world
        .client
        .get(format!(
            "{}/exploitations/{target_id}/entries",
            world.base_url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("get entries request failed");
    world.last_status = Some(response.status().as_u16());
}

#[when(regex = r#"^je demande le dashboard de "([^"]+)" en modifiant l'identifiant dans l'URL$"#)]
async fn when_request_other_dashboard(world: &mut IsolationWorld, target: String) {
    let target_id = world.id_for(&target);
    let current = world
        .current_user
        .clone()
        .expect("no authenticated user set");
    let token = world.token_for(&current);

    let response = world
        .client
        .get(format!(
            "{}/exploitations/{target_id}/dashboard",
            world.base_url
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("get dashboard request failed");
    world.last_status = Some(response.status().as_u16());
}

#[when(regex = r#"^je soumets des coûts pour "([^"]+)" en modifiant l'identifiant dans l'URL$"#)]
async fn when_submit_other_entry(world: &mut IsolationWorld, target: String) {
    let target_id = world.id_for(&target);
    let current = world
        .current_user
        .clone()
        .expect("no authenticated user set");
    let token = world.token_for(&current);
    let mois = chrono::Utc::now().format("%Y-%m").to_string();

    let response = world
        .client
        .post(format!(
            "{}/exploitations/{target_id}/entries",
            world.base_url
        ))
        .bearer_auth(&token)
        .json(&json!({
            "product_id": world.product_id.expect("no product seeded"),
            "mois": mois,
            "quantite": "999",
            "cout": "999",
        }))
        .send()
        .await
        .expect("submit entry request failed");
    world.last_status = Some(response.status().as_u16());
}

#[when("je demande mes propres entries")]
async fn when_request_own_entries(world: &mut IsolationWorld) {
    let current = world
        .current_user
        .clone()
        .expect("no authenticated user set");
    let own_id = world.id_for(&current);
    let token = world.token_for(&current);

    let response = world
        .client
        .get(format!("{}/exploitations/{own_id}/entries", world.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("get entries request failed");
    world.last_status = Some(response.status().as_u16());
}

#[then(regex = r#"^la réponse est refusée avec un statut (\d+)$"#)]
async fn then_response_is_refused(world: &mut IsolationWorld, expected: u16) {
    assert_eq!(
        world.last_status,
        Some(expected),
        "expected the isolation check to reject the request with status {expected}"
    );
}

#[then(regex = r#"^la réponse est acceptée avec un statut (\d+)$"#)]
async fn then_response_is_accepted(world: &mut IsolationWorld, expected: u16) {
    assert_eq!(world.last_status, Some(expected));
}

#[tokio::main]
async fn main() {
    IsolationWorld::cucumber()
        .run("tests/features/isolation.feature")
        .await;
}
