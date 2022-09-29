use super::exchange::Exchange;
use crate::kind::SQLKind;
use crate::model::{self, Commodity};
use futures::executor::block_on;
use std::hash::Hash;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct DataWithPool<T> {
    content: T,
    kind: SQLKind,
    pub pool: sqlx::AnyPool,
}

impl<T> DataWithPool<T> {
    pub(crate) fn new(content: T, kind: SQLKind, pool: sqlx::AnyPool) -> Self
    where
        T: model::NullNone,
    {
        Self {
            content: content.null_none(),
            kind,
            pool,
        }
    }

    pub fn content(&self) -> &T {
        &self.content
    }
}

impl<T> Deref for DataWithPool<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.content
    }
}

impl<T> PartialEq for DataWithPool<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

impl<T> Eq for DataWithPool<T> where T: Eq {}

impl<T> Hash for DataWithPool<T>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.content.hash(state);
    }
}

impl<T> PartialOrd for DataWithPool<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.content.partial_cmp(&other.content)
    }
}

impl DataWithPool<model::Account> {
    pub fn splits(&self) -> Result<Vec<DataWithPool<model::Split>>, sqlx::Error> {
        block_on(async {
            model::Split::query_by_account_guid(&self.guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn parent(&self) -> Option<DataWithPool<model::Account>> {
        let guid = self.parent_guid.as_ref()?;
        block_on(async {
            model::Account::query_by_guid(guid, self.kind)
                .fetch_optional(&self.pool)
                .await
                .unwrap()
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }

    pub fn children(&self) -> Result<Vec<DataWithPool<model::Account>>, sqlx::Error> {
        block_on(async {
            model::Account::query_by_parent_guid(&self.guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn commodity(&self) -> Option<DataWithPool<model::Commodity>> {
        let guid = self.commodity_guid.as_ref()?;
        block_on(async {
            model::Commodity::query_by_guid(guid, self.kind)
                .fetch_optional(&self.pool)
                .await
                .unwrap()
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }

    fn balance_into_currency(
        &self,
        currency: &DataWithPool<Commodity>,
    ) -> Result<f64, sqlx::Error> {
        let mut net: f64 = self.splits()?.iter().map(|s| s.quantity()).sum();
        let commodity = self.commodity().expect("must have commodity");

        for child in self.children()? {
            let child_net = child.balance_into_currency(&commodity)?;
            net += child_net;
        }

        let rate = commodity.sell(currency).unwrap_or_else(|| {
            panic!(
                "must have rate {} to {}",
                commodity.mnemonic, currency.mnemonic
            )
        });
        // dbg!((
        //     &commodity.mnemonic,
        //     &currency.mnemonic,
        //     rate,
        //     &self.name,
        //     net
        // ));

        Ok(net * rate)
    }

    pub fn balance(&self) -> Result<f64, sqlx::Error> {
        let mut net: f64 = self.splits()?.iter().map(|s| s.quantity()).sum();

        let commodity = match self.commodity() {
            Some(commodity) => commodity,
            None => return Ok(net),
        };

        for child in self.children()? {
            let child_net = child.balance_into_currency(&commodity)?;
            net += child_net;
        }
        // dbg!((&self.name, net));

        Ok(net)
    }
}

impl DataWithPool<model::Split> {
    pub fn transaction(&self) -> Result<DataWithPool<model::Transaction>, sqlx::Error> {
        let guid = &self.tx_guid;
        block_on(async {
            model::Transaction::query_by_guid(guid, self.kind)
                .fetch_one(&self.pool)
                .await
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }

    pub fn account(&self) -> Result<DataWithPool<model::Account>, sqlx::Error> {
        let guid = &self.account_guid;
        block_on(async {
            model::Account::query_by_guid(guid, self.kind)
                .fetch_one(&self.pool)
                .await
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }
}

impl DataWithPool<model::Transaction> {
    pub fn currency(&self) -> Result<DataWithPool<model::Commodity>, sqlx::Error> {
        let guid = &self.currency_guid;
        block_on(async {
            model::Commodity::query_by_guid(guid, self.kind)
                .fetch_one(&self.pool)
                .await
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }

    pub fn splits(&self) -> Result<Vec<DataWithPool<model::Split>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Split::query_by_tx_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }
}

impl DataWithPool<model::Price> {
    pub fn commodity(&self) -> Result<DataWithPool<model::Commodity>, sqlx::Error> {
        let guid = &self.commodity_guid;
        block_on(async {
            model::Commodity::query_by_guid(guid, self.kind)
                .fetch_one(&self.pool)
                .await
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }

    pub fn currency(&self) -> Result<DataWithPool<model::Commodity>, sqlx::Error> {
        let guid = &self.currency_guid;
        block_on(async {
            model::Commodity::query_by_guid(guid, self.kind)
                .fetch_one(&self.pool)
                .await
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }
}

impl DataWithPool<model::Commodity> {
    pub fn accounts(&self) -> Result<Vec<DataWithPool<model::Account>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Account::query_by_commodity_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn transactions(&self) -> Result<Vec<DataWithPool<model::Transaction>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Transaction::query_by_currency_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn as_commodity_prices(&self) -> Result<Vec<DataWithPool<model::Price>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Price::query_by_commodity_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn as_currency_prices(&self) -> Result<Vec<DataWithPool<model::Price>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Price::query_by_currency_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn as_commodity_or_currency_prices(
        &self,
    ) -> Result<Vec<DataWithPool<model::Price>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Price::query_by_commodity_or_currency_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn sell(&self, currency: &DataWithPool<model::Commodity>) -> Option<f64> {
        // println!("{} to {}", self.mnemonic, currency.mnemonic);
        let exchange = Exchange::new(self.kind, self.pool.clone()).ok()?;
        exchange.cal(self, currency)
    }

    pub fn buy(&self, commodity: &DataWithPool<model::Commodity>) -> Option<f64> {
        // println!("{} to {}", commodity.mnemonic, self.mnemonic);
        let exchange = Exchange::new(self.kind, self.pool.clone()).ok()?;
        exchange.cal(commodity, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use float_cmp::assert_approx_eq;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        const URI: &str = r"sqlite://tests/db/sqlite/complex_sample.gnucash";
        type DB = sqlx::Sqlite;

        fn setup(uri: &str) -> crate::SqliteBook {
            println!("work_dir: {:?}", std::env::current_dir());
            crate::SqliteBook::new(uri).expect("right path")
        }

        #[test]
        fn account() {
            let book = setup(URI);

            let account = book.account_by_name("Foo stock").unwrap().unwrap();
            assert_eq!("Foo stock", account.name);
            assert_eq!(1, account.splits().unwrap().len());
            assert_eq!("Broker", account.parent().unwrap().name);
            assert_eq!(0, account.children().unwrap().len());
            assert_eq!("FOO", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 130.0, account.balance().unwrap());

            let account = book.account_by_name("Cash").unwrap().unwrap();
            assert_eq!("Cash", account.name);
            assert_eq!(3, account.splits().unwrap().len());
            assert_eq!("Current", account.parent().unwrap().name);
            assert_eq!(0, account.children().unwrap().len());
            assert_eq!("EUR", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 220.0, account.balance().unwrap());

            let account = book.account_by_name("Mouvements").unwrap().unwrap();
            assert_eq!("Mouvements", account.name);
            assert_eq!(0, account.splits().unwrap().len());
            assert_eq!("Root Account", account.parent().unwrap().name);
            assert_eq!(2, account.children().unwrap().len());
            assert_eq!("FOO", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 1351.4815, account.balance().unwrap(), epsilon = 1e-4);

            let account = book.account_by_name("Asset").unwrap().unwrap();
            assert_eq!("Asset", account.name);
            assert_eq!(0, account.splits().unwrap().len());
            assert_eq!("Root Account", account.parent().unwrap().name);
            assert_eq!(3, account.children().unwrap().len());
            assert_eq!("EUR", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 24695.30, account.balance().unwrap());
        }

        #[test]
        fn split() {
            let book = setup(URI);
            let split = book
                .splits()
                .unwrap()
                .into_iter()
                .find(|s| s.guid == "de832fe97e37811a7fff7e28b3a43425")
                .unwrap();

            assert_eq!(
                split.transaction().unwrap().guid,
                "6c8876003c4a6026e38e3afb67d6f2b1"
            );
            assert_eq!(
                split.transaction().unwrap().description,
                Some("income 1".into())
            );
            assert_eq!(
                split.transaction().unwrap().post_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );
            assert_eq!(
                split.transaction().unwrap().enter_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );

            assert_eq!(
                split.account().unwrap().guid,
                "93fc043c3062aaa1297b30e543d2cd0d",
            );
            assert_eq!(split.account().unwrap().name, "Cash",);
        }

        #[test]
        fn transaction() {
            let book = setup(URI);
            let transaction = book
                .transactions()
                .unwrap()
                .into_iter()
                .find(|t| t.description == Some("buy foo".into()))
                .unwrap();

            assert_eq!(transaction.currency().unwrap().mnemonic, "EUR");
            assert_eq!(transaction.splits().unwrap().len(), 4);
        }

        #[test]
        fn price() {
            let book = setup(URI);
            let price = book
                .prices()
                .unwrap()
                .into_iter()
                .find(|p| p.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();

            assert_eq!(price.commodity().unwrap().mnemonic, "ADF");
            assert_eq!(price.currency().unwrap().mnemonic, "AED");
        }

        #[test]
        fn commodity() {
            let book = setup(URI);
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "EUR")
                .unwrap();

            assert_eq!(commodity.accounts().unwrap().len(), 14);
            assert_eq!(commodity.transactions().unwrap().len(), 11);
            assert_eq!(commodity.as_commodity_prices().unwrap().len(), 1);
            assert_eq!(commodity.as_currency_prices().unwrap().len(), 2);
            assert_eq!(
                commodity.as_commodity_or_currency_prices().unwrap().len(),
                3
            );

            let currency = book
                .commodities()
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "FOO")
                .unwrap();
            assert_approx_eq!(f64, 1.2345679012345678, currency.buy(&commodity).unwrap());
            assert_approx_eq!(f64, 1.2345679012345678, commodity.sell(&currency).unwrap());
        }
    }

    #[cfg(feature = "postgresql")]
    mod postgresql {
        use super::*;

        const URI: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
        type DB = sqlx::Postgres;

        fn setup(uri: &str) -> crate::PostgreSQLBook {
            crate::PostgreSQLBook::new(&uri).expect("right path")
        }

        #[test]
        fn account() {
            let book = setup(URI);

            let account = book.account_by_name("Foo stock").unwrap().unwrap();
            assert_eq!("Foo stock", account.name);
            assert_eq!(1, account.splits().unwrap().len());
            assert_eq!("Broker", account.parent().unwrap().name);
            assert_eq!(0, account.children().unwrap().len());
            assert_eq!("FOO", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 130.0, account.balance().unwrap());

            let account = book.account_by_name("Cash").unwrap().unwrap();
            assert_eq!("Cash", account.name);
            assert_eq!(3, account.splits().unwrap().len());
            assert_eq!("Current", account.parent().unwrap().name);
            assert_eq!(0, account.children().unwrap().len());
            assert_eq!("EUR", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 220.0, account.balance().unwrap());

            let account = book.account_by_name("Mouvements").unwrap().unwrap();
            assert_eq!("Mouvements", account.name);
            assert_eq!(0, account.splits().unwrap().len());
            assert_eq!("Root Account", account.parent().unwrap().name);
            assert_eq!(2, account.children().unwrap().len());
            assert_eq!("FOO", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 1351.4815, account.balance().unwrap(), epsilon = 1e-4);

            let account = book.account_by_name("Asset").unwrap().unwrap();
            assert_eq!("Asset", account.name);
            assert_eq!(0, account.splits().unwrap().len());
            assert_eq!("Root Account", account.parent().unwrap().name);
            assert_eq!(3, account.children().unwrap().len());
            assert_eq!("EUR", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 24695.30, account.balance().unwrap());
        }

        #[test]
        fn split() {
            let book = setup(URI);
            let split = book
                .splits()
                .unwrap()
                .into_iter()
                .find(|s| s.guid == "de832fe97e37811a7fff7e28b3a43425")
                .unwrap();

            assert_eq!(
                split.transaction().unwrap().guid,
                "6c8876003c4a6026e38e3afb67d6f2b1"
            );
            assert_eq!(
                split.transaction().unwrap().description,
                Some("income 1".into())
            );
            assert_eq!(
                split.transaction().unwrap().post_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );
            assert_eq!(
                split.transaction().unwrap().enter_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );

            assert_eq!(
                split.account().unwrap().guid,
                "93fc043c3062aaa1297b30e543d2cd0d",
            );
            assert_eq!(split.account().unwrap().name, "Cash",);
        }

        #[test]
        fn transaction() {
            let book = setup(URI);
            let transaction = book
                .transactions()
                .unwrap()
                .into_iter()
                .find(|t| t.description == Some("buy foo".into()))
                .unwrap();

            assert_eq!(transaction.currency().unwrap().mnemonic, "EUR");
            assert_eq!(transaction.splits().unwrap().len(), 4);
        }

        #[test]
        fn price() {
            let book = setup(URI);
            let price = book
                .prices()
                .unwrap()
                .into_iter()
                .find(|p| p.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();

            assert_eq!(price.commodity().unwrap().mnemonic, "ADF");
            assert_eq!(price.currency().unwrap().mnemonic, "AED");
        }

        #[test]
        fn commodity() {
            let book = setup(URI);
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "EUR")
                .unwrap();

            assert_eq!(commodity.accounts().unwrap().len(), 14);
            assert_eq!(commodity.transactions().unwrap().len(), 11);
            assert_eq!(commodity.as_commodity_prices().unwrap().len(), 1);
            assert_eq!(commodity.as_currency_prices().unwrap().len(), 2);
            assert_eq!(
                commodity.as_commodity_or_currency_prices().unwrap().len(),
                3
            );

            let currency = book
                .commodities()
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "FOO")
                .unwrap();
            assert_approx_eq!(f64, 1.2345679012345678, currency.buy(&commodity).unwrap());
            assert_approx_eq!(f64, 1.2345679012345678, commodity.sell(&currency).unwrap());
        }
    }

    #[cfg(feature = "mysql")]
    mod mysql {
        use super::*;

        const URI: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
        type DB = sqlx::MySql;

        fn setup(uri: &str) -> crate::MySQLBook {
            crate::MySQLBook::new(uri).expect("right path")
        }

        #[test]
        fn account() {
            let book = setup(URI);

            let account = book.account_by_name("Foo stock").unwrap().unwrap();
            assert_eq!("Foo stock", account.name);
            assert_eq!(1, account.splits().unwrap().len());
            assert_eq!("Broker", account.parent().unwrap().name);
            assert_eq!(0, account.children().unwrap().len());
            assert_eq!("FOO", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 130.0, account.balance().unwrap());

            let account = book.account_by_name("Cash").unwrap().unwrap();
            assert_eq!("Cash", account.name);
            assert_eq!(3, account.splits().unwrap().len());
            assert_eq!("Current", account.parent().unwrap().name);
            assert_eq!(0, account.children().unwrap().len());
            assert_eq!("EUR", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 220.0, account.balance().unwrap());

            let account = book.account_by_name("Mouvements").unwrap().unwrap();
            assert_eq!("Mouvements", account.name);
            assert_eq!(0, account.splits().unwrap().len());
            assert_eq!("Root Account", account.parent().unwrap().name);
            assert_eq!(2, account.children().unwrap().len());
            assert_eq!("FOO", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 1351.4815, account.balance().unwrap(), epsilon = 1e-4);

            let account = book.account_by_name("Asset").unwrap().unwrap();
            assert_eq!("Asset", account.name);
            assert_eq!(0, account.splits().unwrap().len());
            assert_eq!("Root Account", account.parent().unwrap().name);
            assert_eq!(3, account.children().unwrap().len());
            assert_eq!("EUR", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 24695.30, account.balance().unwrap());
        }

        #[test]
        fn split() {
            let book = setup(URI);
            let split = book
                .splits()
                .unwrap()
                .into_iter()
                .find(|s| s.guid == "de832fe97e37811a7fff7e28b3a43425")
                .unwrap();

            assert_eq!(
                split.transaction().unwrap().guid,
                "6c8876003c4a6026e38e3afb67d6f2b1"
            );
            assert_eq!(
                split.transaction().unwrap().description,
                Some("income 1".into())
            );
            assert_eq!(
                split.transaction().unwrap().post_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );
            assert_eq!(
                split.transaction().unwrap().enter_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );

            assert_eq!(
                split.account().unwrap().guid,
                "93fc043c3062aaa1297b30e543d2cd0d",
            );
            assert_eq!(split.account().unwrap().name, "Cash",);
        }

        #[test]
        fn transaction() {
            let book = setup(URI);
            let transaction = book
                .transactions()
                .unwrap()
                .into_iter()
                .find(|t| t.description == Some("buy foo".into()))
                .unwrap();

            assert_eq!(transaction.currency().unwrap().mnemonic, "EUR");
            assert_eq!(transaction.splits().unwrap().len(), 4);
        }

        #[test]
        fn price() {
            let book = setup(URI);
            let price = book
                .prices()
                .unwrap()
                .into_iter()
                .find(|p| p.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();

            assert_eq!(price.commodity().unwrap().mnemonic, "ADF");
            assert_eq!(price.currency().unwrap().mnemonic, "AED");
        }

        #[test]
        fn commodity() {
            let book = setup(URI);
            let commodity = book
                .commodities()
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "EUR")
                .unwrap();

            assert_eq!(commodity.accounts().unwrap().len(), 14);
            assert_eq!(commodity.transactions().unwrap().len(), 11);
            assert_eq!(commodity.as_commodity_prices().unwrap().len(), 1);
            assert_eq!(commodity.as_currency_prices().unwrap().len(), 2);
            assert_eq!(
                commodity.as_commodity_or_currency_prices().unwrap().len(),
                3
            );

            let currency = book
                .commodities()
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "FOO")
                .unwrap();
            assert_approx_eq!(f64, 1.2345679012345678, currency.buy(&commodity).unwrap());
            assert_approx_eq!(f64, 1.2345679012345678, commodity.sell(&currency).unwrap());
        }
    }
}
