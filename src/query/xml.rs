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
use std::sync::{Arc, Mutex, RwLock};
use std::time::SystemTime;
use tracing::instrument;

use super::Query;
use crate::error::Error;
use account::Account;
use commodity::Commodity;
use price::Price;
use split::Split;
use transaction::Transaction;

type AccountMap = Arc<HashMap<String, Arc<Account>>>;
type AccountsMap = Arc<HashMap<String, Vec<Arc<Account>>>>;

type CommodityMap = Arc<HashMap<String, Arc<Commodity>>>;
type CommoditiesMap = Arc<HashMap<String, Vec<Arc<Commodity>>>>;

type PriceMap = Arc<HashMap<String, Arc<Price>>>;
type PricesMap = Arc<HashMap<String, Vec<Arc<Price>>>>;

type SplitMap = Arc<HashMap<String, Arc<Split>>>;
type SplitsMap = Arc<HashMap<String, Vec<Arc<Split>>>>;

type TransactionMap = Arc<HashMap<String, Arc<Transaction>>>;
type TransactionsMap = Arc<HashMap<String, Vec<Arc<Transaction>>>>;

#[derive(Debug, Default, Clone)]
struct XMLCache {
    accounts: AccountMap,
    commodity_accounts: AccountsMap,
    same_parent_accounts: AccountsMap,
    name_accounts: AccountsMap,
    commodities: CommodityMap,
    namespace_commodities: CommoditiesMap,
    prices: PriceMap,
    commodity_prices: PricesMap,
    currency_prices: PricesMap,
    splits: SplitMap,
    account_splits: SplitsMap,
    transaction_splits: SplitsMap,
    transactions: TransactionMap,
    currency_transactions: TransactionsMap,
}

#[derive(Debug, Clone)]
pub struct XMLQuery {
    file_path: Arc<PathBuf>,
    file_modified_time: Arc<Mutex<SystemTime>>,

    cache: Arc<RwLock<XMLCache>>,
}

impl XMLQuery {
    /// read gnucash xml file in gzip
    #[instrument]
    pub fn new(path: &str) -> Result<Self, Error> {
        tracing::debug!("opening gnucash xml file");
        let path_buf = PathBuf::from_str(path)?;
        let mtime = path_buf.metadata()?.modified()?;

        let cache = Self::load_cache_from_disk(&path_buf)?;

        Ok(Self {
            file_path: Arc::new(path_buf),
            file_modified_time: Arc::new(Mutex::new(mtime)),
            cache: Arc::new(RwLock::new(cache)),
        })
    }

    fn load_cache_from_disk(path: &Path) -> Result<XMLCache, Error> {
        let data = Self::gnucash_data(path)?;
        let doc = Document::parse(&data)?;

        doc.root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .ok_or_else(|| Error::NoBook(path.display().to_string()))?;

        let (acc, acc_c, acc_p, acc_n) = Self::parse_accounts_map(&doc)?;
        let (comm, comm_n) = Self::parse_commodity_map(&doc)?;
        let (prc, prc_c, prc_cur) = Self::parse_price_map(&doc)?;
        let (spl, spl_a, spl_t) = Self::parse_split_map(&doc)?;
        let (txn, txn_c) = Self::parse_transaction_map(&doc)?;

        Ok(XMLCache {
            accounts: acc,
            commodity_accounts: acc_c,
            same_parent_accounts: acc_p,
            name_accounts: acc_n,
            commodities: comm,
            namespace_commodities: comm_n,
            prices: prc,
            commodity_prices: prc_c,
            currency_prices: prc_cur,
            splits: spl,
            account_splits: spl_a,
            transaction_splits: spl_t,
            transactions: txn,
            currency_transactions: txn_c,
        })
    }

    #[instrument(skip(file_path), fields(path = %file_path.display()))]
    fn gnucash_data(file_path: &Path) -> Result<String, Error> {
        tracing::debug!("opening gzip file");
        let f =
            File::open(file_path).inspect_err(|e| tracing::error!("failed to open file: {e}"))?;
        let mut d = GzDecoder::new(f);
        let mut data = String::new();

        tracing::debug!("decompressing gzip data");
        d.read_to_string(&mut data)
            .inspect_err(|e| tracing::error!("failed to decompress data: {e}"))?;

        tracing::debug!(size = data.len(), "gnucash data loaded");
        Ok(data)
    }

    fn parse_accounts_map(
        doc: &Document,
    ) -> Result<(AccountMap, AccountsMap, AccountsMap, AccountsMap), Error> {
        let mut accounts_map = HashMap::new();
        let mut commodity_accounts_map: HashMap<String, Vec<Arc<Account>>> = HashMap::new();
        let mut same_parent_accounts_map: HashMap<String, Vec<Arc<Account>>> = HashMap::new();
        let mut name_accounts_map: HashMap<String, Vec<Arc<Account>>> = HashMap::new();

        let book = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .ok_or_else(|| Error::NoBook("Could not find <book> in document".to_string()))?;

        for n in book.children().filter(|n| n.has_tag_name("account")) {
            let account = Arc::new(Account::try_from(n)?);

            accounts_map.insert(account.guid.clone(), account.clone());

            commodity_accounts_map
                .entry(account.commodity_guid.clone().unwrap_or_default())
                .or_default()
                .push(account.clone());

            same_parent_accounts_map
                .entry(account.parent_guid.clone().unwrap_or_default())
                .or_default()
                .push(account.clone());

            name_accounts_map
                .entry(account.name.clone())
                .or_default()
                .push(account);
        }

        Ok((
            Arc::new(accounts_map),
            Arc::new(commodity_accounts_map),
            Arc::new(same_parent_accounts_map),
            Arc::new(name_accounts_map),
        ))
    }

    fn parse_commodity_map(doc: &Document) -> Result<(CommodityMap, CommoditiesMap), Error> {
        let book = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .ok_or_else(|| Error::NoBook("Could not find <book> in document".to_string()))?;

        let mut commodity_map = HashMap::new();
        let mut namespace_commodities: HashMap<String, Vec<Arc<Commodity>>> = HashMap::new();

        for n in book.children().filter(|n| n.has_tag_name("commodity")) {
            let commodity = Arc::new(Commodity::try_from(n)?);

            commodity_map.insert(commodity.guid.clone(), commodity.clone());
        }

        if let Some(pricedb) = book.children().find(|n| n.has_tag_name("pricedb")) {
            for price in pricedb.children().filter(|n| n.has_tag_name("price")) {
                for child in price.children() {
                    match child.tag_name().name() {
                        "commodity" | "currency" => {
                            let commodity = Arc::new(Commodity::try_from(child)?);

                            commodity_map
                                .entry(commodity.guid.clone())
                                .or_insert(commodity.clone());
                        }

                        _ => {}
                    }
                }
            }
        }

        for c in commodity_map.values() {
            namespace_commodities
                .entry(c.namespace.clone())
                .or_default()
                .push(c.clone());
        }

        Ok((Arc::new(commodity_map), Arc::new(namespace_commodities)))
    }

    fn parse_price_map(doc: &Document) -> Result<(PriceMap, PricesMap, PricesMap), Error> {
        let mut price_map = HashMap::new();
        let mut commodity_prices: HashMap<String, Vec<Arc<Price>>> = HashMap::new();
        let mut currency_prices: HashMap<String, Vec<Arc<Price>>> = HashMap::new();

        let book = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .ok_or_else(|| Error::NoBook("Could not find <book> in document".to_string()))?;

        if let Some(n) = book.children().find(|n| n.has_tag_name("pricedb")) {
            for price in n.children().filter(|n| n.has_tag_name("price")) {
                let price = Arc::new(Price::try_from(price)?);

                price_map.entry(price.guid.clone()).or_insert(price.clone());

                commodity_prices
                    .entry(price.commodity_guid.clone())
                    .or_default()
                    .push(price.clone());

                currency_prices
                    .entry(price.currency_guid.clone())
                    .or_default()
                    .push(price);
            }
        }

        Ok((
            Arc::new(price_map),
            Arc::new(commodity_prices),
            Arc::new(currency_prices),
        ))
    }

    fn parse_split_map(doc: &Document) -> Result<(SplitMap, SplitsMap, SplitsMap), Error> {
        let mut split_map = HashMap::new();
        let mut account_splits_map: HashMap<String, Vec<Arc<Split>>> = HashMap::new();
        let mut transactiont_splits_map: HashMap<String, Vec<Arc<Split>>> = HashMap::new();

        let book = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .ok_or_else(|| Error::NoBook("Could not find <book> in document".to_string()))?;

        for transaction in book.children().filter(|n| n.has_tag_name("transaction")) {
            let tx_guid = transaction
                .children()
                .find(|n| n.has_tag_name("id"))
                .and_then(|n| n.text())
                .map(std::string::ToString::to_string)
                .ok_or_else(|| Error::XMLMissingField {
                    model: "Split".to_string(),
                    field: "tx_guid".to_string(),
                })?;

            for split in transaction
                .children()
                .find(|n| n.has_tag_name("splits"))
                .ok_or_else(|| Error::XMLMissingField {
                    model: "Split".to_string(),
                    field: "splits".to_string(),
                })?
                .children()
                .filter(|n| n.has_tag_name("split"))
            {
                let split = Arc::new(Split::try_from(tx_guid.clone(), split)?);
                split_map.insert(split.guid.clone(), split.clone());

                account_splits_map
                    .entry(split.account_guid.clone())
                    .or_default()
                    .push(split.clone());

                transactiont_splits_map
                    .entry(tx_guid.clone())
                    .or_default()
                    .push(split);
            }
        }

        Ok((
            Arc::new(split_map),
            Arc::new(account_splits_map),
            Arc::new(transactiont_splits_map),
        ))
    }

    fn parse_transaction_map(doc: &Document) -> Result<(TransactionMap, TransactionsMap), Error> {
        let mut transaction_map = HashMap::new();
        let mut currency_transactions_map: HashMap<String, Vec<Arc<Transaction>>> = HashMap::new();

        let book = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .ok_or_else(|| Error::NoBook("Could not find <book> in document".to_string()))?;

        for n in book.children().filter(|n| n.has_tag_name("transaction")) {
            let transaction = Arc::new(Transaction::try_from(n)?);

            transaction_map.insert(transaction.guid.clone(), transaction.clone());

            currency_transactions_map
                .entry(transaction.currency_guid.clone())
                .or_default()
                .push(transaction);
        }

        Ok((
            Arc::new(transaction_map),
            Arc::new(currency_transactions_map),
        ))
    }

    fn is_file_unchanged(&self) -> Result<bool, Error> {
        let current_mtime = self.file_path.metadata()?.modified()?;

        let mut last_mtime = self
            .file_modified_time
            .lock()
            .map_err(|e| Error::Internal(format!("Mtime lock poisoned: {e}")))?;

        if current_mtime == *last_mtime {
            Ok(true)
        } else {
            *last_mtime = current_mtime;
            Ok(false)
        }
    }

    fn update_cache(&self) -> Result<(), Error> {
        if self.is_file_unchanged()? {
            return Ok(());
        }

        let new_cache = Self::load_cache_from_disk(&self.file_path)?;

        let mut cache_lock = self
            .cache
            .write()
            .map_err(|e| Error::Internal(format!("Cache lock poisoned: {e}")))?;

        *cache_lock = new_cache;

        tracing::info!("XML cache updated successfully");
        Ok(())
    }
}

impl Query for XMLQuery {}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    #[test(tokio::test)]
    async fn test_new() {
        let path: &str = &format!(
            "{}/tests/db/xml/complex_sample.gnucash",
            env!("CARGO_MANIFEST_DIR")
        );

        tracing::debug!("work_dir: {:?}", std::env::current_dir());
        XMLQuery::new(path).unwrap();
    }

    #[test(tokio::test)]
    async fn test_is_file_unchanged() {
        let path: &str = &format!(
            "{}/tests/db/xml/complex_sample.gnucash",
            env!("CARGO_MANIFEST_DIR")
        );

        tracing::debug!("work_dir: {:?}", std::env::current_dir());
        let query = XMLQuery::new(path).unwrap();

        assert!(query.is_file_unchanged().unwrap());
    }
}
