use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("XML error: {0}")]
    XML(#[from] xmltree::Error),
    #[error("XML parse error: {0}")]
    XMLParse(#[from] xmltree::ParseError),
    #[error("No book found: {0}")]
    NoBook(String),
    #[error("Exchange graph not available")]
    NoExchangeGraph,
}
