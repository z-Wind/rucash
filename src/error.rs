#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No book found: {0}")]
    NoBook(String),
    #[error("No {model} found for GUID: {guid}")]
    GuidNotFound { model: String, guid: String },
    #[error("Multiple {model} found for GUID: {guid}")]
    GuidMultipleFound { model: String, guid: String },
    #[error("Multiple {model} found for name: {name}")]
    NameMultipleFound { model: String, name: String },
    #[error("Exchange graph not available")]
    NoExchangeGraph,

    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[cfg(any(feature = "postgresql", feature = "mysql"))]
    #[error("SQLx error: {0}")]
    Sql(#[from] sqlx::Error),

    #[cfg(feature = "sqlite")]
    #[error("rusqlite error: {0}")]
    Rusqlite(#[from] rusqlite::Error),

    #[cfg(feature = "xml")]
    #[error("XML error: {0}")]
    XML(#[from] roxmltree::Error),
    #[cfg(feature = "xml")]
    #[error("XML {model} from element")]
    XMLFromElement { model: String },
    #[cfg(feature = "xml")]
    #[error("XML parseInt error: {0}")]
    XMLParseInt(#[from] std::num::ParseIntError),
    #[cfg(feature = "xml")]
    #[error("XML parseNaiveDatetime error: {0}")]
    XMLParseNaiveDatetime(#[from] chrono::ParseError),
}
