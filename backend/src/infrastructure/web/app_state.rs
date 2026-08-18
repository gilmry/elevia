use crate::infrastructure::database::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}
