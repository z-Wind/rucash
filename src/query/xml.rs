pub(crate) mod account;
pub(crate) mod commodity;
pub(crate) mod price;
pub(crate) mod split;
pub(crate) mod transaction;

use flate2::read::GzDecoder;
use roxmltree::Document;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::Query;
use crate::error::Error;
use account::Account;
use commodity::Commodity;
use price::Price;
use split::Split;
use transaction::Transaction;

type AccountMap = Arc<HashMap<String, Account>>;
type CommodityMap = Arc<HashMap<String, Commodity>>;
type PriceMap = Arc<HashMap<String, Price>>;
type SplitMap = Arc<HashMap<String, Split>>;
type TransactionMap = Arc<HashMap<String, Transaction>>;

#[derive(Debug, Clone)]
pub struct XMLQuery {
    file_path: Arc<PathBuf>,
    file_modified_time: Arc<Mutex<SystemTime>>,

    accounts: Arc<Mutex<Option<AccountMap>>>,
    commodities: Arc<Mutex<Option<CommodityMap>>>,
    prices: Arc<Mutex<Option<PriceMap>>>,
    splits: Arc<Mutex<Option<SplitMap>>>,
    transactions: Arc<Mutex<Option<TransactionMap>>>,
}

impl XMLQuery {
    /// read gnucash xml file in gzip
    pub fn new(path: &str) -> Result<Self, Error> {
        let path = PathBuf::from_str(path)?;
        let query = Self {
            file_modified_time: Arc::new(Mutex::new(path.metadata()?.modified()?)),
            file_path: Arc::new(path),

            accounts: Arc::default(),
            commodities: Arc::default(),
            prices: Arc::default(),
            splits: Arc::default(),
            transactions: Arc::default(),
        };

        let data = query.gnucash_data()?;
        let doc = Document::parse(&data)?;
        doc.root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .ok_or(Error::NoBook(query.file_path.display().to_string()))?;

        Ok(query)
    }

    fn gnucash_data(&self) -> Result<String, Error> {
        let f = File::open(&*self.file_path)?;
        let mut d = GzDecoder::new(f);
        let mut data = String::new();
        d.read_to_string(&mut data)?;

        Ok(data)
    }

    fn is_file_unchanged(&self) -> Result<bool, Error> {
        let meta = self.file_path.metadata()?;
        let time = meta.modified()?;

        let mut record_time = self.file_modified_time.lock().unwrap();
        let is_unchanged = time == *record_time;

        if !is_unchanged {
            *record_time = time;
        }

        Ok(is_unchanged)
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

    #[tokio::test]
    async fn test_is_file_unchanged() {
        let path: &str = &format!(
            "{}/tests/db/xml/complex_sample.gnucash",
            env!("CARGO_MANIFEST_DIR")
        );

        println!("work_dir: {:?}", std::env::current_dir());
        let query = XMLQuery::new(path).unwrap();

        assert!(query.is_file_unchanged().unwrap());
    }
}
