use futures::executor::block_on;
use std::rc::Rc;

use super::template::AccountT;
use super::template::Book;
use super::template::BookT;
use super::template::CommodityT;
use super::template::Item;
use super::template::PriceT;
use super::template::SplitT;
use super::template::TransactionT;
use super::template::_Account;
use super::template::_Commodity;
use super::template::_Price;
use super::template::_Split;
use super::template::_Transaction;

type DB = sqlx::Pool<sqlx::Sqlite>;
type Error = Box<dyn std::error::Error>;

pub type Account = Item<_Account, DB>;
pub type Split = Item<_Split, DB>;
pub type Transaction = Item<_Transaction, DB>;
pub type Price = Item<_Price, DB>;
pub type Commodity = Item<_Commodity, DB>;

impl BookT for Book<DB> {
    type DB = DB;
    /// Options and flags which can be used to configure a SQLite connection.
    ///
    /// A value of `SqliteConnectOptions` can be parsed from a connection URI,
    /// as described by [SQLite](https://www.sqlite.org/uri.html).
    ///
    /// | URI | Description |
    /// | -- | -- |
    /// `sqlite::memory:` | Open an in-memory database. |
    /// `sqlite:data.db` | Open the file `data.db` in the current directory. |
    /// `sqlite://data.db` | Open the file `data.db` in the current directory. |
    /// `sqlite:///data.db` | Open the file `data.db` from the root (`/`) directory. |
    /// `sqlite://data.db?mode=ro` | Open the file `data.db` for read-only access. |
    fn new(uri: &str) -> Result<Book<DB>, Error> {
        let pool = block_on(async {
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect(uri)
                .await
        });
        let db = Rc::new(pool?);
        Ok(Book { db })
    }

    fn accounts(&self) -> Result<Vec<Account>, Error> {
        let pool = self.db.as_ref();
        let result = block_on(async { _Account::query().fetch_all(&*pool).await })
            .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());

        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn accounts_contains_name(&self, name: &str) -> Result<Vec<Account>, Error> {
        let name = format!("%{}%", name);
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Account::query_like_name_question_mark(&name)
                .fetch_all(&*pool)
                .await
        })
        .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());

        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn splits(&self) -> Result<Vec<Split>, Error> {
        let pool = self.db.as_ref();
        let result = block_on(async { _Split::query().fetch_all(&*pool).await })
            .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());

        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn transactions(&self) -> Result<Vec<Transaction>, Error> {
        let pool = self.db.as_ref();
        let result = block_on(async { _Transaction::query().fetch_all(&*pool).await })
            .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());

        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn prices(&self) -> Result<Vec<Price>, Error> {
        let pool = self.db.as_ref();
        let result = block_on(async { _Price::query().fetch_all(&*pool).await })
            .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn currencies(&self) -> Result<Vec<Commodity>, Error> {
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Commodity::query_by_namespace_question_mark("CURRENCY")
                .fetch_all(&*pool)
                .await
        })
        .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn commodities(&self) -> Result<Vec<Commodity>, Error> {
        let pool = self.db.as_ref();
        let result = block_on(async { _Commodity::query().fetch_all(&*pool).await })
            .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl AccountT for Account {
    type DB = DB;

    fn splits(&self) -> Result<Vec<Split>, Error> {
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Split::query_by_account_guid_question_mark(&self.guid)
                .fetch_all(&*pool)
                .await
        })
        .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn parent(&self) -> Option<Account> {
        let guid = self.parent_guid.as_ref()?;
        let pool = self.db.as_ref();
        block_on(async {
            _Account::query_by_guid_question_mark(guid)
                .fetch_optional(&*pool)
                .await
                .unwrap()
        })
        .map(|x| Item::new(x, &self.db))
    }

    fn children(&self) -> Result<Vec<Account>, Error> {
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Account::query_by_parent_guid_question_mark(&self.guid)
                .fetch_all(&*pool)
                .await
        })
        .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn commodity(&self) -> Option<Commodity> {
        let guid = self.commodity_guid.as_ref()?;
        let pool = self.db.as_ref();
        block_on(async {
            _Commodity::query_by_guid_question_mark(guid)
                .fetch_optional(&*pool)
                .await
                .unwrap()
        })
        .map(|x| Item::new(x, &self.db))
    }

    fn balance(&self) -> Result<f64, Error> {
        let splits = self.splits()?;
        let mut net = splits.iter().fold(0.0, |acc, x| acc + x.quantity);

        for child in self.children()? {
            let child_net = child.balance()?;
            net += child_net;
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

impl SplitT for Split {
    type DB = DB;

    fn transaction(&self) -> Result<Transaction, Error> {
        let guid = &self.tx_guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Transaction::query_by_guid_question_mark(guid)
                .fetch_one(&*pool)
                .await
        })
        .map(|x| Item::new(x, &self.db));
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn account(&self) -> Result<Account, Error> {
        let guid = &self.account_guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Account::query_by_guid_question_mark(guid)
                .fetch_one(&*pool)
                .await
        })
        .map(|x| Item::new(x, &self.db));
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl TransactionT for Transaction {
    type DB = DB;

    fn currency(&self) -> Result<Commodity, Error> {
        let guid = &self.currency_guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Commodity::query_by_guid_question_mark(guid)
                .fetch_one(&*pool)
                .await
        })
        .map(|x| Item::new(x, &self.db));
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn splits(&self) -> Result<Vec<Split>, Error> {
        let guid = &self.guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Split::query_by_tx_guid_question_mark(guid)
                .fetch_all(&*pool)
                .await
        })
        .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl PriceT for Price {
    type DB = DB;

    fn commodity(&self) -> Result<Commodity, Error> {
        let guid = &self.commodity_guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Commodity::query_by_guid_question_mark(guid)
                .fetch_one(&*pool)
                .await
        })
        .map(|x| Item::new(x, &self.db));
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn currency(&self) -> Result<Commodity, Error> {
        let guid = &self.currency_guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Commodity::query_by_guid_question_mark(guid)
                .fetch_one(&*pool)
                .await
        })
        .map(|x| Item::new(x, &self.db));
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl CommodityT for Commodity {
    type DB = DB;

    fn accounts(&self) -> Result<Vec<Account>, Error> {
        let guid = &self.guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Account::query_by_commodity_guid_question_mark(guid)
                .fetch_all(&*pool)
                .await
        })
        .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn transactions(&self) -> Result<Vec<Transaction>, Error> {
        let guid = &self.guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Transaction::query_by_currency_guid_question_mark(guid)
                .fetch_all(&*pool)
                .await
        })
        .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn as_commodity_prices(&self) -> Result<Vec<Price>, Error> {
        let guid = &self.guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Price::query_by_commodity_guid_question_mark(guid)
                .fetch_all(&*pool)
                .await
        })
        .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn as_currency_prices(&self) -> Result<Vec<Price>, Error> {
        let guid = &self.guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Price::query_by_currency_guid_question_mark(guid)
                .fetch_all(&*pool)
                .await
        })
        .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn as_commodity_or_currency_prices(&self) -> Result<Vec<Price>, Error> {
        let guid = &self.guid;
        let pool = self.db.as_ref();
        let result = block_on(async {
            _Price::query_by_commodity_or_currency_guid_question_mark(guid)
                .fetch_all(&*pool)
                .await
        })
        .map(|v| v.into_iter().map(|x| Item::new(x, &self.db)).collect());
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn sell(&self, currency: &Commodity) -> Result<Option<f64>, Error> {
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
    fn buy(&self, commodity: &Commodity) -> Result<Option<f64>, Error> {
        match commodity.sell(&self) {
            Ok(Some(value)) => Ok(Some(1.0 / value)),
            x => x,
        }
    }
}

// // #[cfg(test)]
// // mod tests {
// //     use super::*;

// //     const URI: &str = "sqlite://tests/db/sqlite/complex_sample.gnucash";
// //     mod book {
// //         use super::*;
// //     }
// //     mod account {
// //         use super::*;
// //     }

// //     mod split {
// //         use super::*;
// //     }

// //     mod transaction {
// //         use super::*;
// //     }

// //     mod price {
// //         use super::*;
// //     }

// //     mod commodity {
// //         use super::*;
// //     }
// // }
