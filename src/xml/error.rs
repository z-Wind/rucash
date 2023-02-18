use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("XML Error: {0}")]
    XML(#[from] xmltree::Error),
    #[error("XML Parse Error: {0}")]
    XMLParse(#[from] xmltree::ParseError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
