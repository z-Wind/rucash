use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("sqlx Error: {0}")]
    SQL(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
