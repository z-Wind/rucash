use futures::executor::block_on;
use std::rc::Rc;

use super::model::account::Account as _Account;
use super::model::commodity::Commodity as _Commodity;
use super::model::price::Price as _Price;
use super::model::split::Split as _Split;
use super::model::transaction::Transaction as _Transaction;
use super::Book;
use super::Item;

type DB = sqlx::Sqlite;
impl Book<DB> {
    /// | URI | Description |
    /// | -- | -- |
    /// sqlite::memory: | Open an in-memory database. |
    /// sqlite:data.db | Open the file data.db in the current directory. |
    /// sqlite://data.db | Open the file data.db in the current directory. |
    /// sqlite:///data.db | Open the file data.db from the root (/) directory. |
    pub fn new(uri: &str) -> Result<Book<DB>, sqlx::Error> {
        let pool = block_on(async {
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect(&format!("{}?mode=ro", uri)) // read only
                .await
        });
        let pool = Rc::new(pool?);
        Ok(Book { pool })
    }

    pub fn accounts(&self) -> Result<Vec<Account>, sqlx::Error> {
        block_on(async { _Account::query().fetch_all(&*self.pool).await }).map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn accounts_contains_name(&self, name: &str) -> Result<Vec<Account>, sqlx::Error> {
        let name = format!("%{}%", name);
        block_on(async {
            _Account::query_like_name(&name)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    fn account_by_name(&self, name: &str) -> Result<Option<Account>, sqlx::Error> {
        let mut v = self.accounts_contains_name(name)?;
        Ok(v.pop())
    }

    fn splits(&self) -> Result<Vec<Split>, sqlx::Error> {
        block_on(async { _Split::query().fetch_all(&*self.pool).await }).map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    fn transactions(&self) -> Result<Vec<Transaction>, sqlx::Error> {
        block_on(async { _Transaction::query().fetch_all(&*self.pool).await }).map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    fn prices(&self) -> Result<Vec<Price>, sqlx::Error> {
        block_on(async { _Price::query().fetch_all(&*self.pool).await }).map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    fn currencies(&self) -> Result<Vec<Commodity>, sqlx::Error> {
        block_on(async {
            _Commodity::query_by_namespace("CURRENCY")
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    fn commodities(&self) -> Result<Vec<Commodity>, sqlx::Error> {
        block_on(async { _Commodity::query().fetch_all(&*self.pool).await }).map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }
}

pub type Account = Item<_Account, DB>;
impl Account {
    fn splits(&self) -> Result<Vec<Split>, sqlx::Error> {
        block_on(async {
            _Split::query_by_account_guid(&self.guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }
    fn parent(&self) -> Option<Account> {
        let guid = self.parent_guid.as_ref()?;
        block_on(async {
            _Account::query_by_guid(guid)
                .fetch_optional(&*self.pool)
                .await
                .unwrap()
        })
        .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
    }
    fn children(&self) -> Result<Vec<Account>, sqlx::Error> {
        block_on(async {
            _Account::query_by_parent_guid(&self.guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }
    fn commodity(&self) -> Option<Commodity> {
        let guid = self.commodity_guid.as_ref()?;
        block_on(async {
            _Commodity::query_by_guid(guid)
                .fetch_optional(&*self.pool)
                .await
                .unwrap()
        })
        .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
    }
    fn balance(&self) -> Result<f64, sqlx::Error> {
        let splits = self.splits()?;
        let mut net = splits.iter().fold(0.0, |acc, x| acc + x.quantity);

        for child in self.children()? {
            let child_net = child.balance()?;
            net = net + child_net;
        }

        let commodity = match self.commodity() {
            Some(commodity) => commodity,
            None => return Ok(net),
        };

        let currency = match self.parent() {
            Some(parent) => match parent.commodity() {
                Some(currency) => currency,
                None => return Ok(net),
            },
            None => return Ok(net),
        };

        match commodity.sell(&currency)? {
            Some(rate) => Ok(net * rate),
            None => Ok(net),
        }
    }
}

pub type Split = Item<_Split, DB>;
impl Split {
    fn transaction(&self) -> Result<Transaction, sqlx::Error> {
        let guid = &self.tx_guid;
        block_on(async {
            _Transaction::query_by_guid(guid)
                .fetch_one(&*self.pool)
                .await
        })
        .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
    }

    fn account(&self) -> Result<Account, sqlx::Error> {
        let guid = &self.account_guid;
        block_on(async { _Account::query_by_guid(guid).fetch_one(&*self.pool).await })
            .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
    }
}

pub type Transaction = Item<_Transaction, DB>;
impl Transaction {
    fn currency(&self) -> Result<Commodity, sqlx::Error> {
        let guid = &self.currency_guid;
        block_on(async { _Commodity::query_by_guid(guid).fetch_one(&*self.pool).await })
            .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
    }

    fn splits(&self) -> Result<Vec<Split>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async { _Split::query_by_tx_guid(guid).fetch_all(&*self.pool).await }).map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }
}

pub type Price = Item<_Price, DB>;
impl Price {
    fn commodity(&self) -> Result<Commodity, sqlx::Error> {
        let guid = &self.commodity_guid;
        block_on(async { _Commodity::query_by_guid(guid).fetch_one(&*self.pool).await })
            .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
    }

    fn currency(&self) -> Result<Commodity, sqlx::Error> {
        let guid = &self.currency_guid;
        block_on(async { _Commodity::query_by_guid(guid).fetch_one(&*self.pool).await })
            .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
    }
}

pub type Commodity = Item<_Commodity, DB>;
impl Commodity {
    fn accounts(&self) -> Result<Vec<Account>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Account::query_by_commodity_guid(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    fn transactions(&self) -> Result<Vec<Transaction>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Transaction::query_by_currency_guid(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    fn as_commodity_prices(&self) -> Result<Vec<Price>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Price::query_by_commodity_guid(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    fn as_currency_prices(&self) -> Result<Vec<Price>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Price::query_by_currency_guid(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    fn as_commodity_or_currency_prices(&self) -> Result<Vec<Price>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Price::query_by_commodity_or_currency_guid(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.iter()
                .map(|x| Item::new(x.clone(), Rc::clone(&self.pool)))
                .collect()
        })
    }

    fn sell(&self, currency: &Commodity) -> Result<Option<f64>, sqlx::Error> {
        if self.guid == currency.guid {
            return Ok(Some(1.0));
        }

        let mut prices: Vec<Price> = self
            .as_commodity_or_currency_prices()?
            .into_iter()
            .filter(|x| x.currency_guid == currency.guid || x.commodity_guid == currency.guid)
            .collect();
        prices.sort_by_key(|x| x.date);

        match prices.last() {
            Some(p) => {
                if self.guid == p.commodity_guid && currency.guid == p.currency_guid {
                    Ok(Some(p.value))
                } else if self.guid == p.currency_guid && currency.guid == p.commodity_guid {
                    Ok(Some(p.value_denom as f64 / p.value_num as f64))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
    fn buy(&self, commodity: &Commodity) -> Result<Option<f64>, sqlx::Error> {
        match commodity.sell(&self) {
            Ok(Some(value)) => Ok(Some(1.0 / value)),
            x => x,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod book {
        use super::*;

        #[test]
        fn new() {
            Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
        }

        #[test]
        #[should_panic]
        fn new_fail() {
            Book::new("sqlite://tests/sample/no.gnucash").unwrap();
        }

        #[test]
        fn accounts_filter() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let accounts: Vec<Account> = book
                .accounts()
                .unwrap()
                .into_iter()
                .filter(|x| x.name.to_lowercase().contains("as"))
                .collect();
            assert_eq!(accounts.len(), 3);
        }

        #[test]
        fn accounts_by_name() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let accounts = book.accounts_contains_name("as").unwrap();
            assert_eq!(accounts.len(), 3);
        }

        #[test]
        fn account_by_name() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let account = book.account_by_name("as").unwrap().unwrap();
            assert_eq!(account.name, "NASDAQ");
        }

        #[test]
        fn splits() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let splits = book.splits().unwrap();
            assert_eq!(splits.len(), 25);
        }

        #[test]
        fn transactions() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let transactions = book.transactions().unwrap();
            assert_eq!(transactions.len(), 11);
        }

        #[test]
        fn prices() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let prices = book.prices().unwrap();
            assert_eq!(prices.len(), 5);
        }

        #[test]
        fn commodities() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let commodities = book.commodities().unwrap();
            assert_eq!(commodities.len(), 5);
        }

        #[test]
        fn currencies() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let currencies = book.currencies().unwrap();
            assert_eq!(currencies.len(), 4);
        }
    }
    mod account {
        use super::*;
        #[test]
        fn balance() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let account = book
                .accounts()
                .unwrap()
                .into_iter()
                .filter(|x| x.name == "Current")
                .next()
                .unwrap();

            assert_eq!(account.balance().unwrap(), 4590.0);
        }
        #[test]
        fn balance_diff_currency() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let account = book
                .accounts()
                .unwrap()
                .into_iter()
                .filter(|x| x.name == "Asset")
                .next()
                .unwrap();

            assert_eq!(account.balance().unwrap(), 24695.3);
        }
        #[test]
        fn splits() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let account = book.account_by_name("Cash").unwrap().unwrap();
            let splits = account.splits().unwrap();
            assert_eq!(splits.len(), 3);
        }

        #[test]
        fn parent() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let account = book.account_by_name("Cash").unwrap().unwrap();
            let parent = account.parent().unwrap();
            assert_eq!(parent.name, "Current");
        }

        #[test]
        fn no_parent() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let account = book.account_by_name("Root Account").unwrap().unwrap();
            let parent = account.parent();
            assert!(parent.is_none());
        }

        #[test]
        fn children() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let account = book.account_by_name("Current").unwrap().unwrap();
            let children = account.children().unwrap();
            assert_eq!(children.len(), 3);
        }

        #[test]
        fn commodity() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let account = book.account_by_name("Cash").unwrap().unwrap();
            let commodity = account.commodity().unwrap();
            assert_eq!(commodity.mnemonic, "EUR");
        }
    }

    mod split {
        use super::*;
        #[test]
        fn transaction() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let split = book
                .splits()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
                .next()
                .unwrap();
            let transaction = split.transaction().unwrap();
            assert_eq!(transaction.description.as_ref().unwrap(), "income 1");
        }

        #[test]
        fn account() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let split = book
                .splits()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
                .next()
                .unwrap();
            let account = split.account().unwrap();
            assert_eq!(account.name, "Cash");
        }
    }

    mod transaction {
        use super::*;

        #[test]
        fn currency() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let transaction = book
                .transactions()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .next()
                .unwrap();
            let currency = transaction.currency().unwrap();
            assert_eq!(currency.fullname.as_ref().unwrap(), "Euro");
        }

        #[test]
        fn splits() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let transaction = book
                .transactions()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .next()
                .unwrap();
            let splits = transaction.splits().unwrap();
            assert_eq!(splits.len(), 2);
        }
    }

    mod price {
        use super::*;

        #[test]
        fn commodity() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let price = book
                .prices()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
                .next()
                .unwrap();
            let commodity = price.commodity().unwrap();
            assert_eq!(commodity.fullname.as_ref().unwrap(), "Andorran Franc");
        }

        #[test]
        fn currency() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let price = book
                .prices()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
                .next()
                .unwrap();
            let currency = price.currency().unwrap();
            assert_eq!(currency.fullname.as_ref().unwrap(), "UAE Dirham");
        }
    }

    mod commodity {
        use super::*;

        #[test]
        fn accounts() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .next()
                .unwrap();
            let accounts = commodity.accounts().unwrap();
            assert_eq!(accounts.len(), 14);
        }

        #[test]
        fn transactions() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .next()
                .unwrap();
            let transactions = commodity.transactions().unwrap();
            assert_eq!(transactions.len(), 11);
        }

        #[test]
        fn as_commodity_prices() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .next()
                .unwrap();
            let prices = commodity.as_commodity_prices().unwrap();
            assert_eq!(prices.len(), 1);
        }

        #[test]
        fn as_currency_prices() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .next()
                .unwrap();
            let prices = commodity.as_currency_prices().unwrap();
            assert_eq!(prices.len(), 2);
        }

        #[test]
        fn as_commodity_or_currency_prices() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .next()
                .unwrap();
            let prices = commodity.as_commodity_or_currency_prices().unwrap();
            assert_eq!(prices.len(), 3);
        }

        #[test]
        fn rate_direct() {
            // ADF => AED
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "d821d6776fde9f7c2d01b67876406fd3")
                .next()
                .unwrap();
            let currency = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .next()
                .unwrap();

            let rate = commodity.sell(&currency).unwrap().unwrap();
            assert_eq!(rate, 1.5);
            let rate = currency.buy(&commodity).unwrap().unwrap();
            assert_eq!(rate, 1.0 / 1.5);

            // AED => EUR
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .next()
                .unwrap();
            let currency = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .next()
                .unwrap();

            let rate = commodity.sell(&currency).unwrap().unwrap();
            assert_eq!(rate, 9.0 / 10.0);
            let rate = currency.buy(&commodity).unwrap().unwrap();
            assert_eq!(rate, 10.0 / 9.0);
        }

        #[test]
        fn rate_indirect() {
            let book = Book::new("sqlite://tests/sample/complex_sample.gnucash").unwrap();
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "1e5d65e2726a5d4595741cb204992991")
                .next()
                .unwrap();
            let currency = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .next()
                .unwrap();

            let rate = commodity.sell(&currency).unwrap();
            assert_eq!(rate, None);
            // assert_eq!(rate, 7.0 / 5.0 * 10.0 / 9.0);
        }
    }
}
