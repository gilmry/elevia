pub mod app_state;
pub mod handlers;
pub mod routes;

pub use app_state::AppState;
pub use routes::configure_routes;

// middleware (AuthenticatedUser / ExploitationId JWT extractors, mirroring the
// isolation guarantees required by the spec) lands here alongside the auth use case.
