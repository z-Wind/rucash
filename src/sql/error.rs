use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("SQLx error: {0}")]
    SQL(#[from] sqlx::Error),

    #[error("Commodity GUID does not exist")]
    NoCommodityGuid,
    #[error("Commodity not found for GUID: {0}")]
    CommodityNotFound(String),
    #[error("Exchange graph not available")]
    NoExchangeGraph,
}
