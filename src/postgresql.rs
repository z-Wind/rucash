use futures::executor::block_on;
use std::rc::Rc;

use super::model::account::Account as _Account;
use super::model::commodity::Commodity as _Commodity;
use super::model::price::Price as _Price;
use super::model::split::Split as _Split;
use super::model::transaction::Transaction as _Transaction;
use super::Book;
use super::Item;

type DB = sqlx::Postgres;
pub type Account = Item<_Account, DB>;
pub type Split = Item<_Split, DB>;
pub type Transaction = Item<_Transaction, DB>;
pub type Price = Item<_Price, DB>;
pub type Commodity = Item<_Commodity, DB>;

impl Book<DB> {
    /// The general form for a connection URI is:
    ///
    /// ```text
    /// postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
    /// ```
    ///
    /// ## Parameters
    ///
    /// |Parameter|Default|Description|
    /// |---------|-------|-----------|
    /// | `sslmode` | `prefer` | Determines whether or with what priority a secure SSL TCP/IP connection will be negotiated. |
    /// | `sslrootcert` | `None` | Sets the name of a file containing a list of trusted SSL Certificate Authorities. |
    /// | `statement-cache-capacity` | `100` | The maximum number of prepared statements stored in the cache. Set to `0` to disable. |
    /// | `host` | `None` | Path to the directory containing a PostgreSQL unix domain socket, which will be used instead of TCP if set. |
    /// | `hostaddr` | `None` | Same as `host`, but only accepts IP addresses. |
    /// | `application-name` | `None` | The name will be displayed in the pg_stat_activity view and included in CSV log entries. |
    /// | `user` | result of `whoami` | PostgreSQL user name to connect as. |
    /// | `password` | `None` | Password to be used if the server demands password authentication. |
    /// | `port` | `5432` | Port number to connect to at the server host, or socket file name extension for Unix-domain connections. |
    /// | `dbname` | `None` | The database name. |
    ///
    /// The URI scheme designator can be either `postgresql://` or `postgres://`.
    /// Each of the URI parts is optional.
    ///
    /// ```text
    /// postgresql://
    /// postgresql://localhost
    /// postgresql://localhost:5433
    /// postgresql://localhost/mydb
    /// postgresql://user@localhost
    /// postgresql://user:secret@localhost
    /// postgresql://localhost?dbname=mydb&user=postgres&password=postgres
    /// ```
    pub fn new(uri: &str) -> Result<Book<DB>, sqlx::Error> {
        let pool = block_on(async {
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(uri)
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
            _Account::query_like_name_money_mark(&name)
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
            _Commodity::query_by_namespace_money_mark("CURRENCY")
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

impl Account {
    pub fn splits(&self) -> Result<Vec<Split>, sqlx::Error> {
        block_on(async {
            _Split::query_by_account_guid_money_mark(&self.guid)
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
            _Account::query_by_guid_money_mark(guid)
                .fetch_optional(&*self.pool)
                .await
                .unwrap()
        })
        .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }
    pub fn children(&self) -> Result<Vec<Account>, sqlx::Error> {
        block_on(async {
            _Account::query_by_parent_guid_money_mark(&self.guid)
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
            _Commodity::query_by_guid_money_mark(guid)
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

impl Split {
    pub fn transaction(&self) -> Result<Transaction, sqlx::Error> {
        let guid = &self.tx_guid;
        block_on(async {
            _Transaction::query_by_guid_money_mark(guid)
                .fetch_one(&*self.pool)
                .await
        })
        .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }

    pub fn account(&self) -> Result<Account, sqlx::Error> {
        let guid = &self.account_guid;
        block_on(async {
            _Account::query_by_guid_money_mark(guid)
                .fetch_one(&*self.pool)
                .await
        })
        .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }
}

impl Transaction {
    pub fn currency(&self) -> Result<Commodity, sqlx::Error> {
        let guid = &self.currency_guid;
        block_on(async {
            _Commodity::query_by_guid_money_mark(guid)
                .fetch_one(&*self.pool)
                .await
        })
        .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }

    pub fn splits(&self) -> Result<Vec<Split>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Split::query_by_tx_guid_money_mark(guid)
                .fetch_all(&*self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| Item::new(x, Rc::clone(&self.pool)))
                .collect()
        })
    }
}

impl Price {
    pub fn commodity(&self) -> Result<Commodity, sqlx::Error> {
        let guid = &self.commodity_guid;
        block_on(async {
            _Commodity::query_by_guid_money_mark(guid)
                .fetch_one(&*self.pool)
                .await
        })
        .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }

    pub fn currency(&self) -> Result<Commodity, sqlx::Error> {
        let guid = &self.currency_guid;
        block_on(async {
            _Commodity::query_by_guid_money_mark(guid)
                .fetch_one(&*self.pool)
                .await
        })
        .map(|x| Item::new(x, Rc::clone(&self.pool)))
    }
}

impl Commodity {
    pub fn accounts(&self) -> Result<Vec<Account>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            _Account::query_by_commodity_guid_money_mark(guid)
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
            _Transaction::query_by_currency_guid_money_mark(guid)
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
            _Price::query_by_commodity_guid_money_mark(guid)
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
            _Price::query_by_currency_guid_money_mark(guid)
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
            _Price::query_by_commodity_or_currency_guid_money_mark(guid)
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