pub(crate) mod account;
pub(crate) mod commodity;
pub(crate) mod price;
pub(crate) mod split;
pub(crate) mod transaction;

use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use xmltree::Element;

use super::Query;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct XMLQuery {
    tree: Arc<Element>,
}

impl XMLQuery {
    /// read gnucash xml file in gzip
    pub fn new(path: &str) -> Result<Self, Error> {
        let f = File::open(path)?;
        let mut d = GzDecoder::new(f);
        let mut data = String::new();
        d.read_to_string(&mut data)?;

        let mut root: Element = Element::parse(data.as_bytes())?;
        root = root
            .take_child("book")
            .ok_or(Error::NoBook(path.to_string()))?;

        Ok(Self {
            tree: Arc::new(root),
        })
    }
}

impl Query for XMLQuery {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new() {
        let path: &str = &format!(
            "{}/tests/db/xml/complex_sample.gnucash",
            env!("CARGO_MANIFEST_DIR")
        );

        println!("work_dir: {:?}", std::env::current_dir());
        XMLQuery::new(path).unwrap();
    }
}
