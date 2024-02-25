#[cfg(feature = "xml")]
use std::num::ParseIntError;

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

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
    #[error("SQLx error: {0}")]
    Sql(#[from] sqlx::Error),

    #[cfg(feature = "xml")]
    #[error("XML error: {0}")]
    XML(#[from] xmltree::Error),
    #[cfg(feature = "xml")]
    #[error("XML parse error: {0}")]
    XMLParse(#[from] xmltree::ParseError),
    #[cfg(feature = "xml")]
    #[error("XML {model} from element")]
    XMLFromElement { model: String },
    #[cfg(feature = "xml")]
    #[error("XML parseInt error: {0}")]
    XMLParseInt(#[from] ParseIntError),
    #[cfg(feature = "xml")]
    #[error("XML parseNaiveDatetime error: {0}")]
    XMLParseNaiveDatetime(#[from] chrono::ParseError),
    #[cfg(feature = "xml")]
    #[error("XML no splits: {0}")]
    XMLNoSplit(String),
}
