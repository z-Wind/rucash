use super::exchange::Exchange;
use crate::kind::SQLKind;
use crate::model::{self, Commodity};
use crate::SQLError;
use futures::future::{BoxFuture, FutureExt};
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct DataWithPool<T> {
    content: T,
    kind: SQLKind,
    pub pool: sqlx::AnyPool,
    exchange_graph: Option<Arc<RwLock<Exchange>>>,
}

impl<T> DataWithPool<T> {
    pub(crate) fn new(
        content: T,
        kind: SQLKind,
        pool: sqlx::AnyPool,
        exchange_graph: Option<Arc<RwLock<Exchange>>>,
    ) -> Self
    where
        T: model::NullNone,
    {
        Self {
            content: content.null_none(),
            kind,
            pool,
            exchange_graph,
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
    pub async fn splits(&self) -> Result<Vec<DataWithPool<model::Split>>, SQLError> {
        model::Split::query_by_account_guid(&self.guid, self.kind)
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
            .map_err(std::convert::Into::into)
    }

    pub async fn parent(&self) -> Result<Option<DataWithPool<model::Account>>, SQLError> {
        let Some(ref guid )= self.parent_guid else{
            return Ok(None);
        };

        Ok(model::Account::query_by_guid(guid, self.kind)
            .fetch_optional(&self.pool)
            .await?
            .map(|x| {
                DataWithPool::new(x, self.kind, self.pool.clone(), self.exchange_graph.clone())
            }))
    }

    pub async fn children(&self) -> Result<Vec<DataWithPool<model::Account>>, SQLError> {
        model::Account::query_by_parent_guid(&self.guid, self.kind)
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
            .map_err(std::convert::Into::into)
    }

    pub async fn commodity(&self) -> Result<DataWithPool<model::Commodity>, SQLError> {
        let guid = self
            .commodity_guid
            .as_ref()
            .ok_or(SQLError::NoCommodityGuid)?;

        model::Commodity::query_by_guid(guid, self.kind)
            .fetch_optional(&self.pool)
            .await?
            .map(|x| {
                DataWithPool::new(x, self.kind, self.pool.clone(), self.exchange_graph.clone())
            })
            .ok_or(SQLError::CommodityNotFound(guid.clone()))
            .map_err(std::convert::Into::into)
    }

    fn balance_into_currency<'a>(
        &'a self,
        currency: &'a DataWithPool<Commodity>,
    ) -> BoxFuture<'a, Result<crate::Num, SQLError>> {
        async move {
            let mut net: crate::Num = self.splits().await?.iter().map(|s| s.quantity()).sum();
            let commodity = self.commodity().await.expect("must have commodity");

            for child in self.children().await? {
                let child_net = child.balance_into_currency(&commodity).await?;
                net += child_net;
            }

            let rate = commodity.sell(currency).await.unwrap_or_else(|| {
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
        .boxed()
    }

    pub async fn balance(&self) -> Result<crate::Num, SQLError> {
        let mut net: crate::Num = self.splits().await?.iter().map(|s| s.quantity()).sum();

        let commodity = self.commodity().await?;

        for child in self.children().await? {
            let child_net = child.balance_into_currency(&commodity).await?;
            net += child_net;
        }
        // dbg!((&self.name, net));

        Ok(net)
    }
}

impl DataWithPool<model::Split> {
    pub async fn transaction(&self) -> Result<DataWithPool<model::Transaction>, SQLError> {
        let guid = &self.tx_guid;

        model::Transaction::query_by_guid(guid, self.kind)
            .fetch_one(&self.pool)
            .await
            .map(|x| {
                DataWithPool::new(x, self.kind, self.pool.clone(), self.exchange_graph.clone())
            })
            .map_err(std::convert::Into::into)
    }

    pub async fn account(&self) -> Result<DataWithPool<model::Account>, SQLError> {
        let guid = &self.account_guid;

        model::Account::query_by_guid(guid, self.kind)
            .fetch_one(&self.pool)
            .await
            .map(|x| {
                DataWithPool::new(x, self.kind, self.pool.clone(), self.exchange_graph.clone())
            })
            .map_err(std::convert::Into::into)
    }
}

impl DataWithPool<model::Transaction> {
    pub async fn currency(&self) -> Result<DataWithPool<model::Commodity>, SQLError> {
        let guid = &self.currency_guid;

        model::Commodity::query_by_guid(guid, self.kind)
            .fetch_one(&self.pool)
            .await
            .map(|x| {
                DataWithPool::new(x, self.kind, self.pool.clone(), self.exchange_graph.clone())
            })
            .map_err(std::convert::Into::into)
    }

    pub async fn splits(&self) -> Result<Vec<DataWithPool<model::Split>>, SQLError> {
        let guid = &self.guid;

        model::Split::query_by_tx_guid(guid, self.kind)
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
            .map_err(std::convert::Into::into)
    }
}

impl DataWithPool<model::Price> {
    pub async fn commodity(&self) -> Result<DataWithPool<model::Commodity>, SQLError> {
        let guid = &self.commodity_guid;

        model::Commodity::query_by_guid(guid, self.kind)
            .fetch_one(&self.pool)
            .await
            .map(|x| {
                DataWithPool::new(x, self.kind, self.pool.clone(), self.exchange_graph.clone())
            })
            .map_err(std::convert::Into::into)
    }

    pub async fn currency(&self) -> Result<DataWithPool<model::Commodity>, SQLError> {
        let guid = &self.currency_guid;

        model::Commodity::query_by_guid(guid, self.kind)
            .fetch_one(&self.pool)
            .await
            .map(|x| {
                DataWithPool::new(x, self.kind, self.pool.clone(), self.exchange_graph.clone())
            })
            .map_err(std::convert::Into::into)
    }
}

impl DataWithPool<model::Commodity> {
    pub async fn accounts(&self) -> Result<Vec<DataWithPool<model::Account>>, SQLError> {
        let guid = &self.guid;

        model::Account::query_by_commodity_guid(guid, self.kind)
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
            .map_err(std::convert::Into::into)
    }

    pub async fn transactions(&self) -> Result<Vec<DataWithPool<model::Transaction>>, SQLError> {
        let guid = &self.guid;

        model::Transaction::query_by_currency_guid(guid, self.kind)
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
            .map_err(std::convert::Into::into)
    }

    pub async fn as_commodity_prices(&self) -> Result<Vec<DataWithPool<model::Price>>, SQLError> {
        let guid = &self.guid;

        model::Price::query_by_commodity_guid(guid, self.kind)
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
            .map_err(std::convert::Into::into)
    }

    pub async fn as_currency_prices(&self) -> Result<Vec<DataWithPool<model::Price>>, SQLError> {
        let guid = &self.guid;

        model::Price::query_by_currency_guid(guid, self.kind)
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
            .map_err(std::convert::Into::into)
    }

    pub async fn as_commodity_or_currency_prices(
        &self,
    ) -> Result<Vec<DataWithPool<model::Price>>, SQLError> {
        let guid = &self.guid;

        model::Price::query_by_commodity_or_currency_guid(guid, self.kind)
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
            .map_err(std::convert::Into::into)
    }

    pub async fn sell(&self, currency: &DataWithPool<model::Commodity>) -> Option<crate::Num> {
        // println!("{} to {}", self.mnemonic, currency.mnemonic);
        self.exchange_graph
            .as_ref()?
            .read()
            .await
            .cal(self, currency)
    }

    pub async fn buy(&self, commodity: &DataWithPool<model::Commodity>) -> Option<crate::Num> {
        commodity.sell(self).await
    }

    pub async fn update_exchange_graph(&self) -> Result<(), SQLError> {
        let graph = self
            .exchange_graph
            .as_ref()
            .ok_or(SQLError::NoExchangeGraph)?;

        graph.write().await.update().await
    }
}

#[cfg(test)]
mod tests {
    //use super::*;use pretty_assertions::assert_eq;
    use chrono::NaiveDateTime;
    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;
        use pretty_assertions::assert_eq;

        //type DB = sqlx::Sqlite;

        async fn setup() -> crate::SqliteBook {
            let uri: &str = &format!(
                "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );
            crate::SqliteBook::new(uri)
                .await
                .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"))
        }

        #[tokio::test]
        async fn account() {
            let book = setup().await;

            let account = book.account_by_name("Foo stock").await.unwrap().unwrap();
            assert_eq!("Foo stock", account.name);
            assert_eq!(1, account.splits().await.unwrap().len());
            assert_eq!("Broker", account.parent().await.unwrap().unwrap().name);
            assert_eq!(0, account.children().await.unwrap().len());
            assert_eq!("FOO", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 130.0, account.balance().await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(130, 0), account.balance().await.unwrap());

            let account = book.account_by_name("Cash").await.unwrap().unwrap();
            assert_eq!("Cash", account.name);
            assert_eq!(3, account.splits().await.unwrap().len());
            assert_eq!("Current", account.parent().await.unwrap().unwrap().name);
            assert_eq!(0, account.children().await.unwrap().len());
            assert_eq!("EUR", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 220.0, account.balance().await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(220, 0), account.balance().await.unwrap());

            let account = book.account_by_name("Mouvements").await.unwrap().unwrap();
            assert_eq!("Mouvements", account.name);
            assert_eq!(0, account.splits().await.unwrap().len());
            assert_eq!(
                "Root Account",
                account.parent().await.unwrap().unwrap().name
            );
            assert_eq!(2, account.children().await.unwrap().len());
            assert_eq!("FOO", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(
                f64,
                1351.4815,
                account.balance().await.unwrap(),
                epsilon = 1e-4
            );
            #[cfg(feature = "decimal")]
            assert_eq!(
                Decimal::new(13514815, 4),
                account.balance().await.unwrap().round_dp(4)
            );

            let account = book.account_by_name("Asset").await.unwrap().unwrap();
            assert_eq!("Asset", account.name);
            assert_eq!(0, account.splits().await.unwrap().len());
            assert_eq!(
                "Root Account",
                account.parent().await.unwrap().unwrap().name
            );
            assert_eq!(3, account.children().await.unwrap().len());
            assert_eq!("EUR", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 24695.3, account.balance().await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(2469530, 2), account.balance().await.unwrap());
        }

        #[tokio::test]
        async fn split() {
            let book = setup().await;
            let split = book
                .splits()
                .await
                .unwrap()
                .into_iter()
                .find(|s| s.guid == "de832fe97e37811a7fff7e28b3a43425")
                .unwrap();

            assert_eq!(
                split.transaction().await.unwrap().guid,
                "6c8876003c4a6026e38e3afb67d6f2b1"
            );
            assert_eq!(
                split.transaction().await.unwrap().description,
                Some("income 1".into())
            );
            assert_eq!(
                split.transaction().await.unwrap().post_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );
            assert_eq!(
                split.transaction().await.unwrap().enter_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );

            assert_eq!(
                split.account().await.unwrap().guid,
                "93fc043c3062aaa1297b30e543d2cd0d",
            );
            assert_eq!(split.account().await.unwrap().name, "Cash",);
        }

        #[tokio::test]
        async fn transaction() {
            let book = setup().await;
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|t| t.description == Some("buy foo".into()))
                .unwrap();

            assert_eq!(transaction.currency().await.unwrap().mnemonic, "EUR");
            assert_eq!(transaction.splits().await.unwrap().len(), 4);
        }

        #[tokio::test]
        async fn price() {
            let book = setup().await;
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|p| p.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();

            assert_eq!(price.commodity().await.unwrap().mnemonic, "ADF");
            assert_eq!(price.currency().await.unwrap().mnemonic, "AED");
        }

        #[tokio::test]
        async fn commodity() {
            let book = setup().await;
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "EUR")
                .unwrap();

            assert_eq!(commodity.accounts().await.unwrap().len(), 14);
            assert_eq!(commodity.transactions().await.unwrap().len(), 11);
            assert_eq!(commodity.as_commodity_prices().await.unwrap().len(), 1);
            assert_eq!(commodity.as_currency_prices().await.unwrap().len(), 2);
            assert_eq!(
                commodity
                    .as_commodity_or_currency_prices()
                    .await
                    .unwrap()
                    .len(),
                3
            );

            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "FOO")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0 / 0.81, currency.buy(&commodity).await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(
                Decimal::new(1, 0) / Decimal::new(81, 2),
                currency.buy(&commodity).await.unwrap()
            );

            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0 / 0.81, commodity.sell(&currency).await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(
                Decimal::new(1, 0) / Decimal::new(81, 2),
                commodity.sell(&currency).await.unwrap()
            );
        }
    }

    #[cfg(feature = "postgresql")]
    mod postgresql {
        use super::*;
        use pretty_assertions::assert_eq;

        //type DB = sqlx::Postgres;

        async fn setup() -> crate::PostgreSQLBook {
            let uri: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            crate::PostgreSQLBook::new(&uri)
                .await
                .unwrap_or_else(|e| panic!("{} uri:{:?}", e, uri))
        }

        #[tokio::test]
        async fn account() {
            let book = setup().await;

            let account = book.account_by_name("Foo stock").await.unwrap().unwrap();
            assert_eq!("Foo stock", account.name);
            assert_eq!(1, account.splits().await.unwrap().len());
            assert_eq!("Broker", account.parent().await.unwrap().name);
            assert_eq!(0, account.children().await.unwrap().len());
            assert_eq!("FOO", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 130.0, account.balance().await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(130, 0), account.balance().await.unwrap());

            let account = book.account_by_name("Cash").await.unwrap().unwrap();
            assert_eq!("Cash", account.name);
            assert_eq!(3, account.splits().await.unwrap().len());
            assert_eq!("Current", account.parent().await.unwrap().name);
            assert_eq!(0, account.children().await.unwrap().len());
            assert_eq!("EUR", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 220.0, account.balance().await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(220, 0), account.balance().await.unwrap());

            let account = book.account_by_name("Mouvements").await.unwrap().unwrap();
            assert_eq!("Mouvements", account.name);
            assert_eq!(0, account.splits().await.unwrap().len());
            assert_eq!("Root Account", account.parent().await.unwrap().name);
            assert_eq!(2, account.children().await.unwrap().len());
            assert_eq!("FOO", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(
                f64,
                1351.4815,
                account.balance().await.unwrap(),
                epsilon = 1e-4
            );
            #[cfg(feature = "decimal")]
            assert_eq!(
                Decimal::new(13514815, 4),
                account.balance().await.unwrap().round_dp(4)
            );

            let account = book.account_by_name("Asset").await.unwrap().unwrap();
            assert_eq!("Asset", account.name);
            assert_eq!(0, account.splits().await.unwrap().len());
            assert_eq!("Root Account", account.parent().await.unwrap().name);
            assert_eq!(3, account.children().await.unwrap().len());
            assert_eq!("EUR", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 24695.3, account.balance().await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(2469530, 2), account.balance().await.unwrap());
        }

        #[tokio::test]
        async fn split() {
            let book = setup().await;
            let split = book
                .splits()
                .await
                .unwrap()
                .into_iter()
                .find(|s| s.guid == "de832fe97e37811a7fff7e28b3a43425")
                .unwrap();

            assert_eq!(
                split.transaction().await.unwrap().guid,
                "6c8876003c4a6026e38e3afb67d6f2b1"
            );
            assert_eq!(
                split.transaction().await.unwrap().description,
                Some("income 1".into())
            );
            assert_eq!(
                split.transaction().await.unwrap().post_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );
            assert_eq!(
                split.transaction().await.unwrap().enter_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );

            assert_eq!(
                split.account().await.unwrap().guid,
                "93fc043c3062aaa1297b30e543d2cd0d",
            );
            assert_eq!(split.account().await.unwrap().name, "Cash",);
        }

        #[tokio::test]
        async fn transaction() {
            let book = setup().await;
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|t| t.description == Some("buy foo".into()))
                .unwrap();

            assert_eq!(transaction.currency().await.unwrap().mnemonic, "EUR");
            assert_eq!(transaction.splits().await.unwrap().len(), 4);
        }

        #[tokio::test]
        async fn price() {
            let book = setup().await;
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|p| p.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();

            assert_eq!(price.commodity().unwrap().mnemonic, "ADF");
            assert_eq!(price.currency().unwrap().mnemonic, "AED");
        }

        #[tokio::test]
        async fn commodity() {
            let book = setup();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "EUR")
                .unwrap();

            assert_eq!(commodity.accounts().await.unwrap().len(), 14);
            assert_eq!(commodity.transactions().await.unwrap().len(), 11);
            assert_eq!(commodity.as_commodity_prices().await.unwrap().len(), 1);
            assert_eq!(commodity.as_currency_prices().await.unwrap().len(), 2);
            assert_eq!(
                commodity
                    .as_commodity_or_currency_prices()
                    .await
                    .unwrap()
                    .len(),
                3
            );

            let currency = book
                .commodities()
                .await
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
        use pretty_assertions::assert_eq;

        //type DB = sqlx::MySql;

        async fn setup() -> crate::MySQLBook {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            crate::MySQLBook::new(uri)
                .await
                .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"))
        }

        #[tokio::test]
        async fn account() {
            let book = setup().await;

            let account = book.account_by_name("Foo stock").await.unwrap().unwrap();
            assert_eq!("Foo stock", account.name);
            assert_eq!(1, account.splits().await.unwrap().len());
            assert_eq!("Broker", account.parent().await.unwrap().unwrap().name);
            assert_eq!(0, account.children().await.unwrap().len());
            assert_eq!("FOO", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 130.0, account.balance().await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(130, 0), account.balance().await.unwrap());

            let account = book.account_by_name("Cash").await.unwrap().unwrap();
            assert_eq!("Cash", account.name);
            assert_eq!(3, account.splits().await.unwrap().len());
            assert_eq!("Current", account.parent().await.unwrap().unwrap().name);
            assert_eq!(0, account.children().await.unwrap().len());
            assert_eq!("EUR", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 220.0, account.balance().await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(220, 0), account.balance().await.unwrap());

            let account = book.account_by_name("Mouvements").await.unwrap().unwrap();
            assert_eq!("Mouvements", account.name);
            assert_eq!(0, account.splits().await.unwrap().len());
            assert_eq!(
                "Root Account",
                account.parent().await.unwrap().unwrap().name
            );
            assert_eq!(2, account.children().await.unwrap().len());
            assert_eq!("FOO", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(
                f64,
                1351.4815,
                account.balance().await.unwrap(),
                epsilon = 1e-4
            );
            #[cfg(feature = "decimal")]
            assert_eq!(
                Decimal::new(13514815, 4),
                account.balance().await.unwrap().round_dp(4)
            );

            let account = book.account_by_name("Asset").await.unwrap().unwrap();
            assert_eq!("Asset", account.name);
            assert_eq!(0, account.splits().await.unwrap().len());
            assert_eq!(
                "Root Account",
                account.parent().await.unwrap().unwrap().name
            );
            assert_eq!(3, account.children().await.unwrap().len());
            assert_eq!("EUR", account.commodity().await.unwrap().mnemonic);
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 24695.3, account.balance().await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(2469530, 2), account.balance().await.unwrap());
        }

        #[tokio::test]
        async fn split() {
            let book = setup().await;
            let split = book
                .splits()
                .await
                .unwrap()
                .into_iter()
                .find(|s| s.guid == "de832fe97e37811a7fff7e28b3a43425")
                .unwrap();

            assert_eq!(
                split.transaction().await.unwrap().guid,
                "6c8876003c4a6026e38e3afb67d6f2b1"
            );
            assert_eq!(
                split.transaction().await.unwrap().description,
                Some("income 1".into())
            );
            assert_eq!(
                split.transaction().await.unwrap().post_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );
            assert_eq!(
                split.transaction().await.unwrap().enter_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );

            assert_eq!(
                split.account().await.unwrap().guid,
                "93fc043c3062aaa1297b30e543d2cd0d",
            );
            assert_eq!(split.account().await.unwrap().name, "Cash",);
        }

        #[tokio::test]
        async fn transaction() {
            let book = setup().await;
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|t| t.description == Some("buy foo".into()))
                .unwrap();

            assert_eq!(transaction.currency().await.unwrap().mnemonic, "EUR");
            assert_eq!(transaction.splits().await.unwrap().len(), 4);
        }

        #[tokio::test]
        async fn price() {
            let book = setup().await;
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|p| p.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();

            assert_eq!(price.commodity().await.unwrap().mnemonic, "ADF");
            assert_eq!(price.currency().await.unwrap().mnemonic, "AED");
        }

        #[tokio::test]
        async fn commodity() {
            let book = setup().await;
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "EUR")
                .unwrap();

            assert_eq!(commodity.accounts().await.unwrap().len(), 14);
            assert_eq!(commodity.transactions().await.unwrap().len(), 11);
            assert_eq!(commodity.as_commodity_prices().await.unwrap().len(), 1);
            assert_eq!(commodity.as_currency_prices().await.unwrap().len(), 2);
            assert_eq!(
                commodity
                    .as_commodity_or_currency_prices()
                    .await
                    .unwrap()
                    .len(),
                3
            );

            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|p| p.mnemonic == "FOO")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0 / 0.81, currency.buy(&commodity).await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(
                Decimal::new(1, 0) / Decimal::new(81, 2),
                currency.buy(&commodity).await.unwrap()
            );

            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0 / 0.81, commodity.sell(&currency).await.unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(
                Decimal::new(1, 0) / Decimal::new(81, 2),
                commodity.sell(&currency).await.unwrap()
            );
        }
    }
}
