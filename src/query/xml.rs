pub(crate) mod account;
pub(crate) mod commodity;
pub(crate) mod price;
pub(crate) mod split;
pub(crate) mod transaction;

use flate2::read::GzDecoder;
use roxmltree::Document;
use std::fs::File;
use std::io::Read;

use super::Query;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct XMLQuery {
    text: String,
}

impl XMLQuery {
    /// read gnucash xml file in gzip
    pub fn new(path: &str) -> Result<Self, Error> {
        let f = File::open(path)?;
        let mut d = GzDecoder::new(f);
        let mut data = String::new();
        d.read_to_string(&mut data)?;

        let doc = Document::parse(&data)?;
        doc.root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .ok_or(Error::NoBook(path.to_string()))?;

        Ok(Self { text: data })
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
