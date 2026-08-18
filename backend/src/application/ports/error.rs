use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("resource not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
