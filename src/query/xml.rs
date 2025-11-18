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
use std::path::{Path, PathBuf};
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
type SplitMap = Arc<HashMap<String, Arc<Split>>>;
type TransactionMap = Arc<HashMap<String, Transaction>>;
type AccountSplitsMap = Arc<HashMap<String, Vec<Arc<Split>>>>;
type TransactionSplitsMap = Arc<HashMap<String, Vec<Arc<Split>>>>;

#[derive(Debug, Clone)]
pub struct XMLQuery {
    file_path: Arc<PathBuf>,
    file_modified_time: Arc<Mutex<SystemTime>>,

    accounts: Arc<Mutex<AccountMap>>,
    commodities: Arc<Mutex<CommodityMap>>,
    prices: Arc<Mutex<PriceMap>>,
    splits: Arc<Mutex<SplitMap>>,
    transactions: Arc<Mutex<TransactionMap>>,

    account_splits: Arc<Mutex<AccountSplitsMap>>,
    transaction_splits: Arc<Mutex<TransactionSplitsMap>>,
}

impl XMLQuery {
    /// read gnucash xml file in gzip
    pub fn new(path: &str) -> Result<Self, Error> {
        let path = PathBuf::from_str(path)?;
        let data = Self::gnucash_data(&path)?;
        let doc = Document::parse(&data)?;

        let accounts = Self::parse_account_map(&doc)?;
        let commodities = Self::parse_commodity_map(&doc)?;
        let prices = Self::parse_price_map(&doc)?;
        let (splits, account_splits, transaction_splits) = Self::parse_split_map(&doc)?;
        let transactions = Self::parse_transaction_map(&doc)?;

        let query = Self {
            file_modified_time: Arc::new(Mutex::new(path.metadata()?.modified()?)),
            file_path: Arc::new(path),

            accounts: Arc::new(Mutex::new(accounts)),
            commodities: Arc::new(Mutex::new(commodities)),
            prices: Arc::new(Mutex::new(prices)),
            splits: Arc::new(Mutex::new(splits)),
            transactions: Arc::new(Mutex::new(transactions)),

            account_splits: Arc::new(Mutex::new(account_splits)),
            transaction_splits: Arc::new(Mutex::new(transaction_splits)),
        };

        doc.root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .ok_or(Error::NoBook(query.file_path.display().to_string()))?;

        Ok(query)
    }

    fn gnucash_data(file_path: &Path) -> Result<String, Error> {
        let f = File::open(file_path)?;
        let mut d = GzDecoder::new(f);
        let mut data = String::new();
        d.read_to_string(&mut data)?;

        Ok(data)
    }

    fn parse_account_map(doc: &Document) -> Result<AccountMap, Error> {
        let accounts = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .expect("must exist book")
            .children()
            .filter(|n| n.has_tag_name("account"))
            .map(|n| {
                let result = Account::try_from(n);

                result.map(|a| (a.guid.clone(), a))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Arc::new(accounts))
    }

    fn parse_commodity_map(doc: &Document) -> Result<CommodityMap, Error> {
        let book = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .expect("must exist book");

        let mut commodities: HashMap<String, Commodity> = book
            .children()
            .filter(|n| n.has_tag_name("commodity"))
            .map(|n| {
                let result = Commodity::try_from(n);
                result.map(|c| (c.guid.clone(), c))
            })
            .collect::<Result<_, _>>()?;

        if let Some(pricedb) = book.children().find(|n| n.has_tag_name("pricedb")) {
            for price in pricedb.children().filter(|n| n.has_tag_name("price")) {
                for child in price.children() {
                    match child.tag_name().name() {
                        "commodity" | "currency" => {
                            let commodity = Commodity::try_from(child)?;
                            commodities
                                .entry(commodity.guid.clone())
                                .or_insert(commodity);
                        }

                        _ => {}
                    }
                }
            }
        }

        Ok(Arc::new(commodities))
    }

    fn parse_price_map(doc: &Document) -> Result<PriceMap, Error> {
        let prices = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .expect("must exist book")
            .children()
            .find(|n| n.has_tag_name("pricedb"))
            .map_or_else(
                || Ok(HashMap::new()),
                |n| {
                    n.children()
                        .filter(|n| n.has_tag_name("price"))
                        .map(|n| {
                            let result = Price::try_from(n);
                            result.map(|p| (p.guid.clone(), p))
                        })
                        .collect()
                },
            )?;

        Ok(Arc::new(prices))
    }

    fn parse_split_map(
        doc: &Document,
    ) -> Result<(SplitMap, AccountSplitsMap, TransactionSplitsMap), Error> {
        let mut splits = HashMap::new();
        let mut account_splits: HashMap<String, Vec<Arc<Split>>> = HashMap::new();
        let mut transactiont_splits: HashMap<String, Vec<Arc<Split>>> = HashMap::new();

        for transaction in doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .expect("must exist book")
            .children()
            .filter(|n| n.has_tag_name("transaction"))
        {
            let tx_guid = transaction
                .children()
                .find(|n| n.has_tag_name("id"))
                .and_then(|n| n.text())
                .map(std::string::ToString::to_string)
                .ok_or(Error::XMLFromElement {
                    model: "Split no tx_guid".to_string(),
                })?;

            for split in transaction
                .children()
                .find(|n| n.has_tag_name("splits"))
                .ok_or(Error::XMLFromElement {
                    model: "Split no child splits".to_string(),
                })?
                .children()
                .filter(|n| n.has_tag_name("split"))
            {
                let split = Arc::new(Split::try_from(tx_guid.clone(), split)?);
                splits.insert(split.guid.clone(), split.clone());

                account_splits
                    .entry(split.account_guid.clone())
                    .or_default()
                    .push(split.clone());

                transactiont_splits
                    .entry(tx_guid.clone())
                    .or_default()
                    .push(split);
            }
        }

        Ok((
            Arc::new(splits),
            Arc::new(account_splits),
            Arc::new(transactiont_splits),
        ))
    }

    fn parse_transaction_map(doc: &Document) -> Result<TransactionMap, Error> {
        let transactions = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .expect("must exist book")
            .children()
            .filter(|n| n.has_tag_name("transaction"))
            .map(|n| {
                let result = Transaction::try_from(n);
                result.map(|t| (t.guid.clone(), t))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Arc::new(transactions))
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

    fn update_cache(&self) -> Result<(), Error> {
        if self.is_file_unchanged()? {
            return Ok(());
        }

        let data = Self::gnucash_data(&self.file_path)?;
        let doc = Document::parse(&data)?;

        {
            *self.accounts.lock().unwrap() = Self::parse_account_map(&doc)?;
        }
        {
            *self.commodities.lock().unwrap() = Self::parse_commodity_map(&doc)?;
        }
        {
            *self.prices.lock().unwrap() = Self::parse_price_map(&doc)?;
        }
        {
            (
                *self.splits.lock().unwrap(),
                *self.account_splits.lock().unwrap(),
                *self.transaction_splits.lock().unwrap(),
            ) = Self::parse_split_map(&doc)?;
        }
        {
            *self.transactions.lock().unwrap() = Self::parse_transaction_map(&doc)?;
        }

        Ok(())
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
