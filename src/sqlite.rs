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
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
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
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn account_by_name(&self, name: &str) -> Result<Option<Account>, sqlx::Error> {
        let mut v = self.accounts_contains_name(name)?;
        Ok(v.pop())
    }

    pub fn splits(&self) -> Result<Vec<Split>, sqlx::Error> {
        block_on(async { _Split::query().fetch_all(&*self.pool).await }).map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn transactions(&self) -> Result<Vec<Transaction>, sqlx::Error> {
        block_on(async { _Transaction::query().fetch_all(&*self.pool).await }).map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn prices(&self) -> Result<Vec<Price>, sqlx::Error> {
        block_on(async { _Price::query().fetch_all(&*self.pool).await }).map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn currencies(&self) -> Result<Vec<Commodity>, sqlx::Error> {
        block_on(async {
            _Commodity::query_by_namespace("CURRENCY")
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn commodities(&self) -> Result<Vec<Commodity>, sqlx::Error> {
        block_on(async { _Commodity::query().fetch_all(&*self.pool).await }).map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }
}

pub type Account = Item<_Account, DB>;
impl Account {
    pub fn splits(&self) -> Result<Vec<Split>, sqlx::Error> {
        block_on(async {
            _Split::query_by_account_guid(&self.guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }
    pub fn parent(&self) -> Option<Account> {
        let guid = self.parent_guid.as_ref()?;
        block_on(async {
            _Account::query_by_guid(guid)
                .fetch_optional(&*self.pool)
                .await
                .unwrap()
        })
        .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }
    pub fn children(&self) -> Result<Vec<Account>, sqlx::Error> {
        block_on(async {
            _Account::query_by_parent_guid(&self.guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }
    pub fn commodity(&self) -> Option<Commodity> {
        let guid = self.commodity_guid.as_ref()?;
        block_on(async {
            _Commodity::query_by_guid(guid)
                .fetch_optional(&*self.pool)
                .await
                .unwrap()
        })
        .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }
    pub fn balance(&self) -> Result<f64, sqlx::Error> {
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

pub type Split = Item<_Split, DB>;
impl Split {
    pub fn transaction(&self) -> Result<Transaction, sqlx::Error> {
        let guid = &self.tx_guid;
        block_on(async {
            _Transaction::query_by_guid(guid)
                .fetch_one(&*self.pool)
                .await
        })
        .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }

    pub fn account(&self) -> Result<Account, sqlx::Error> {
        let guid = &self.account_guid;
        block_on(async { _Account::query_by_guid(guid).fetch_one(&*self.pool).await })
            .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }
}

pub type Transaction = Item<_Transaction, DB>;
impl Transaction {
    pub fn currency(&self) -> Result<Commodity, sqlx::Error> {
        let guid = &self.currency_guid;
        block_on(async { _Commodity::query_by_guid(guid).fetch_one(&*self.pool).await })
            .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }

    pub fn splits(&self) -> Result<Vec<Split>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async { _Split::query_by_tx_guid(guid).fetch_all(&*self.pool).await }).map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }
}

pub type Price = Item<_Price, DB>;
impl Price {
    pub fn commodity(&self) -> Result<Commodity, sqlx::Error> {
        let guid = &self.commodity_guid;
        block_on(async { _Commodity::query_by_guid(guid).fetch_one(&*self.pool).await })
            .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }

    pub fn currency(&self) -> Result<Commodity, sqlx::Error> {
        let guid = &self.currency_guid;
        block_on(async { _Commodity::query_by_guid(guid).fetch_one(&*self.pool).await })
            .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }
}

pub type Commodity = Item<_Commodity, DB>;
impl Commodity {
    pub fn accounts(&self) -> Result<Vec<Account>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Account::query_by_commodity_guid(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn transactions(&self) -> Result<Vec<Transaction>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Transaction::query_by_currency_guid(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn as_commodity_prices(&self) -> Result<Vec<Price>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Price::query_by_commodity_guid(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn as_currency_prices(&self) -> Result<Vec<Price>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Price::query_by_currency_guid(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn as_commodity_or_currency_prices(&self) -> Result<Vec<Price>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Price::query_by_commodity_or_currency_guid(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }

    pub fn sell(&self, currency: &Commodity) -> Result<Option<f64>, sqlx::Error> {
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
    pub fn buy(&self, commodity: &Commodity) -> Result<Option<f64>, sqlx::Error> {
        match commodity.sell(&self) {
            Ok(Some(value)) => Ok(Some(1.0 / value)),
            x => x,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     const URI: &str = "sqlite://tests/sqlite/sample/complex_sample.gnucash";
//     mod book {
//         use super::*;
//     }
//     mod account {
//         use super::*;
//     }

//     mod split {
//         use super::*;
//     }

//     mod transaction {
//         use super::*;
//     }

//     mod price {
//         use super::*;
//     }

//     mod commodity {
//         use super::*;
//     }
// }
