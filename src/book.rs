use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::instrument;

use crate::error::Error;
use crate::exchange::Exchange;
use crate::model::{Account, Commodity, Price, Split, Transaction};
use crate::query::Query;

#[derive(Debug, Clone)]
pub struct Book<Q>
where
    Q: Query,
{
    pub(crate) query: Arc<Q>,
    pub(crate) exchange_graph: Option<Arc<Mutex<Exchange>>>,
}

impl<Q> Book<Q>
where
    Q: Query,
{
    #[instrument(skip(query))]
    pub async fn new(query: Q) -> Result<Self, Error> {
        tracing::debug!("creating new book instance");
        let query = Arc::new(query);
        let mut book = Self {
            query,
            exchange_graph: None,
        };

        book.update_exchange_graph()
            .await
            .inspect_err(|e| tracing::error!("failed to update exchange graph: {e}"))?;
        tracing::info!("book created successfully");
        Ok(book)
    }

    #[instrument(skip(self))]
    pub async fn accounts(&self) -> Result<Vec<Account<Q>>, Error> {
        tracing::debug!("fetching all accounts");
        let accounts = self
            .query
            .accounts()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch accounts: {e}"))?;
        let result: Vec<_> = accounts
            .into_iter()
            .map(|x| Account::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::info!(count = result.len(), "accounts fetched successfully");
        Ok(result)
    }

    #[instrument(skip(self))]
    pub async fn accounts_contains_name_ignore_case(
        &self,
        name: &str,
    ) -> Result<Vec<Account<Q>>, Error> {
        tracing::debug!("searching accounts containing name: {name}");
        let accounts = self
            .query
            .accounts_contains_name_ignore_case(name)
            .await
            .inspect_err(|e| tracing::error!("failed to search accounts: {e}"))?;
        let result: Vec<_> = accounts
            .into_iter()
            .map(|x| Account::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::info!(count = result.len(), "found accounts matching name pattern");
        Ok(result)
    }

    #[instrument(skip(self))]
    pub async fn account_contains_name_ignore_case(
        &self,
        name: &str,
    ) -> Result<Option<Account<Q>>, Error> {
        let mut accounts = self.accounts_contains_name_ignore_case(name).await?;
        match accounts.pop() {
            None => {
                tracing::debug!("no account found matching name: {name}");
                Ok(None)
            }
            Some(x) if accounts.is_empty() => {
                tracing::info!("found single account matching name: {name}");
                Ok(Some(x))
            }
            _ => {
                tracing::warn!("multiple accounts found for name: {name}");
                Err(Error::NameMultipleFound {
                    model: "Account".to_string(),
                    name: name.to_string(),
                })
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn splits(&self) -> Result<Vec<Split<Q>>, Error> {
        tracing::debug!("fetching all splits");
        let splits = self
            .query
            .splits()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch splits: {e}"))?;
        let result: Vec<_> = splits
            .into_iter()
            .map(|x| Split::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::info!(count = result.len(), "splits fetched successfully");
        Ok(result)
    }

    #[instrument(skip(self))]
    pub async fn transactions(&self) -> Result<Vec<Transaction<Q>>, Error> {
        tracing::debug!("fetching all transactions");
        let transactions = self
            .query
            .transactions()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch transactions: {e}"))?;
        let result: Vec<_> = transactions
            .into_iter()
            .map(|x| Transaction::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::info!(count = result.len(), "transactions fetched successfully");
        Ok(result)
    }

    #[instrument(skip(self))]
    pub async fn prices(&self) -> Result<Vec<Price<Q>>, Error> {
        tracing::debug!("fetching all prices");
        let prices = self
            .query
            .prices()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch prices: {e}"))?;
        let result: Vec<_> = prices
            .into_iter()
            .map(|x| Price::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::info!(count = result.len(), "prices fetched successfully");
        Ok(result)
    }

    #[instrument(skip(self))]
    pub async fn commodities(&self) -> Result<Vec<Commodity<Q>>, Error> {
        tracing::debug!("fetching all commodities");
        let commodities = self
            .query
            .commodities()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch commodities: {e}"))?;
        let result: Vec<_> = commodities
            .into_iter()
            .map(|x| Commodity::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::info!(count = result.len(), "commodities fetched successfully");
        Ok(result)
    }

    #[instrument(skip(self))]
    pub async fn currencies(&self) -> Result<Vec<Commodity<Q>>, Error> {
        tracing::debug!("fetching all currencies");
        let currencies = self
            .query
            .currencies()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch currencies: {e}"))?;
        let result: Vec<_> = currencies
            .into_iter()
            .map(|x| Commodity::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::info!(count = result.len(), "currencies fetched successfully");
        Ok(result)
    }

    #[instrument(skip(self, commodity, currency), fields(
        commodity_mnemonic = %commodity.mnemonic,
        currency_mnemonic = %currency.mnemonic
    ))]
    pub async fn exchange(
        &self,
        commodity: &Commodity<Q>,
        currency: &Commodity<Q>,
    ) -> Option<crate::Num> {
        tracing::debug!("calculating exchange rate");
        let graph = self.exchange_graph.as_ref()?;
        let result = graph.lock().await.cal(commodity, currency);
        if let Some(rate) = result {
            tracing::info!(?rate, "exchange rate calculated");
            Some(rate)
        } else {
            tracing::warn!("no exchange rate path found");
            None
        }
    }

    #[instrument(skip(self))]
    pub async fn update_exchange_graph(&mut self) -> Result<(), Error> {
        match &self.exchange_graph {
            None => {
                tracing::debug!("initializing exchange graph");
                let exchange_graph = Exchange::new(self.query.clone())
                    .await
                    .inspect_err(|e| tracing::error!("failed to create exchange graph: {e}"))?;
                self.exchange_graph = Some(Arc::new(Mutex::new(exchange_graph)));
                tracing::info!("exchange graph initialized");
                Ok(())
            }
            Some(graph) => {
                tracing::debug!("updating existing exchange graph");
                graph
                    .lock()
                    .await
                    .update(self.query.clone())
                    .await
                    .inspect_err(|e| tracing::error!("failed to update exchange graph: {e}"))?;
                tracing::info!("exchange graph updated");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;
    use tokio::sync::OnceCell;

    use super::*;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::SQLiteQuery;

        use super::*;

        static Q: OnceCell<Book<SQLiteQuery>> = OnceCell::const_new();
        async fn setup() -> &'static Book<SQLiteQuery> {
            Q.get_or_init(|| async {
                let uri: &str = &format!(
                    "{}/tests/db/sqlite/complex_sample.gnucash",
                    env!("CARGO_MANIFEST_DIR")
                );

                tracing::info!("work_dir: {:?}", std::env::current_dir());
                let query = SQLiteQuery::new(uri).unwrap();
                Book::new(query).await.unwrap()
            })
            .await
        }

        #[test(tokio::test)]
        async fn test_new() {
            let uri: &str = &format!(
                "{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );
            let query = SQLiteQuery::new(uri).unwrap();
            Book::new(query).await.unwrap();
        }

        #[test(tokio::test)]
        async fn test_new_fail() {
            assert!(matches!(
                SQLiteQuery::new("tests/sample/no.gnucash"),
                Err(crate::Error::Rusqlite(_))
            ));
        }

        #[test(tokio::test)]
        async fn test_accounts() {
            let book = setup().await;
            let accounts = book.accounts().await.unwrap();
            assert_eq!(accounts.len(), 21);
        }

        #[test(tokio::test)]
        async fn test_accounts_contains_name() {
            let book = setup().await;
            let accounts = book.accounts_contains_name_ignore_case("aS").await.unwrap();
            assert_eq!(accounts.len(), 3);
        }

        #[test(tokio::test)]
        async fn test_account_contains_name() {
            let book = setup().await;
            let account = book
                .account_contains_name_ignore_case("NAS")
                .await
                .unwrap()
                .unwrap();
            assert_eq!(account.name, "NASDAQ");
        }

        #[test(tokio::test)]
        async fn test_splits() {
            let book = setup().await;
            let splits = book.splits().await.unwrap();
            assert_eq!(splits.len(), 25);
        }

        #[test(tokio::test)]
        async fn test_transactions() {
            let book = setup().await;
            let transactions = book.transactions().await.unwrap();
            assert_eq!(transactions.len(), 11);
        }

        #[test(tokio::test)]
        async fn test_prices() {
            let book = setup().await;
            let prices = book.prices().await.unwrap();
            assert_eq!(prices.len(), 5);
        }

        #[test(tokio::test)]
        async fn test_commodities() {
            let book = setup().await;
            let commodities = book.commodities().await.unwrap();
            assert_eq!(commodities.len(), 5);
        }

        #[test(tokio::test)]
        async fn test_currencies() {
            let book = setup().await;
            let currencies = book.currencies().await.unwrap();
            assert_eq!(currencies.len(), 4);
        }

        #[test(tokio::test)]
        async fn test_exchange() {
            let book = setup().await;
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "d821d6776fde9f7c2d01b67876406fd3")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .unwrap();

            let rate = book.exchange(&commodity, &currency).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));
        }
    }

    #[cfg(feature = "mysql")]
    mod mysql {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::MySQLQuery;

        use super::*;

        static Q: OnceCell<Book<MySQLQuery>> = OnceCell::const_new();
        async fn setup() -> &'static Book<MySQLQuery> {
            Q.get_or_init(|| async {
                let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
                let query = MySQLQuery::new(uri)
                    .await
                    .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"));
                Book::new(query).await.unwrap()
            })
            .await
        }

        #[test(tokio::test)]
        async fn test_new() {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            let query = MySQLQuery::new(uri).await.unwrap();
            Book::new(query).await.unwrap();
        }

        #[test(tokio::test)]
        async fn test_new_fail() {
            assert!(matches!(
                MySQLQuery::new("mysql://tests/sample/no.gnucash").await,
                Err(crate::Error::Sql(_))
            ));
        }

        #[test(tokio::test)]
        async fn test_accounts() {
            let book = setup().await;
            let accounts = book.accounts().await.unwrap();
            assert_eq!(accounts.len(), 21);
        }

        #[test(tokio::test)]
        async fn test_accounts_contains_name() {
            let book = setup().await;
            let accounts = book.accounts_contains_name_ignore_case("aS").await.unwrap();
            assert_eq!(accounts.len(), 3);
        }

        #[test(tokio::test)]
        async fn test_account_contains_name() {
            let book = setup().await;
            let account = book
                .account_contains_name_ignore_case("NAS")
                .await
                .unwrap()
                .unwrap();
            assert_eq!(account.name, "NASDAQ");
        }

        #[test(tokio::test)]
        async fn test_splits() {
            let book = setup().await;
            let splits = book.splits().await.unwrap();
            assert_eq!(splits.len(), 25);
        }

        #[test(tokio::test)]
        async fn test_transactions() {
            let book = setup().await;
            let transactions = book.transactions().await.unwrap();
            assert_eq!(transactions.len(), 11);
        }

        #[test(tokio::test)]
        async fn test_prices() {
            let book = setup().await;
            let prices = book.prices().await.unwrap();
            assert_eq!(prices.len(), 5);
        }

        #[test(tokio::test)]
        async fn test_commodities() {
            let book = setup().await;
            let commodities = book.commodities().await.unwrap();
            assert_eq!(commodities.len(), 5);
        }

        #[test(tokio::test)]
        async fn test_currencies() {
            let book = setup().await;
            let currencies = book.currencies().await.unwrap();
            assert_eq!(currencies.len(), 4);
        }

        #[test(tokio::test)]
        async fn test_exchange() {
            let book = setup().await;
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "d821d6776fde9f7c2d01b67876406fd3")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .unwrap();

            let rate = book.exchange(&commodity, &currency).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));
        }
    }

    #[cfg(feature = "postgresql")]
    mod postgresql {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::PostgreSQLQuery;

        use super::*;

        static Q: OnceCell<Book<PostgreSQLQuery>> = OnceCell::const_new();
        async fn setup() -> &'static Book<PostgreSQLQuery> {
            Q.get_or_init(|| async {
                let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
                let query = PostgreSQLQuery::new(uri)
                    .await
                    .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"));
                Book::new(query).await.unwrap()
            })
            .await
        }

        #[test(tokio::test)]
        async fn test_new() {
            let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            let query = PostgreSQLQuery::new(uri).await.unwrap();
            Book::new(query).await.unwrap();
        }

        #[test(tokio::test)]
        async fn test_new_fail() {
            assert!(matches!(
                PostgreSQLQuery::new("postgresql://tests/sample/no.gnucash").await,
                Err(crate::Error::Sql(_))
            ));
        }

        #[test(tokio::test)]
        async fn test_accounts() {
            let book = setup().await;
            let accounts = book.accounts().await.unwrap();
            assert_eq!(accounts.len(), 21);
        }

        #[test(tokio::test)]
        async fn test_accounts_contains_name() {
            let book = setup().await;
            let accounts = book.accounts_contains_name_ignore_case("aS").await.unwrap();
            assert_eq!(accounts.len(), 3);
        }

        #[test(tokio::test)]
        async fn test_account_contains_name() {
            let book = setup().await;
            let account = book
                .account_contains_name_ignore_case("NAS")
                .await
                .unwrap()
                .unwrap();
            assert_eq!(account.name, "NASDAQ");
        }

        #[test(tokio::test)]
        async fn test_splits() {
            let book = setup().await;
            let splits = book.splits().await.unwrap();
            assert_eq!(splits.len(), 25);
        }

        #[test(tokio::test)]
        async fn test_transactions() {
            let book = setup().await;
            let transactions = book.transactions().await.unwrap();
            assert_eq!(transactions.len(), 11);
        }

        #[test(tokio::test)]
        async fn test_prices() {
            let book = setup().await;
            let prices = book.prices().await.unwrap();
            assert_eq!(prices.len(), 5);
        }

        #[test(tokio::test)]
        async fn test_commodities() {
            let book = setup().await;
            let commodities = book.commodities().await.unwrap();
            assert_eq!(commodities.len(), 5);
        }

        #[test(tokio::test)]
        async fn test_currencies() {
            let book = setup().await;
            let currencies = book.currencies().await.unwrap();
            assert_eq!(currencies.len(), 4);
        }

        #[test(tokio::test)]
        async fn test_exchange() {
            let book = setup().await;
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "d821d6776fde9f7c2d01b67876406fd3")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .unwrap();

            let rate = book.exchange(&commodity, &currency).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));
        }
    }

    #[cfg(feature = "xml")]
    mod xml {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::XMLQuery;

        use super::*;

        static Q: OnceCell<Book<XMLQuery>> = OnceCell::const_new();
        async fn setup() -> &'static Book<XMLQuery> {
            Q.get_or_init(|| async {
                let path: &str = &format!(
                    "{}/tests/db/xml/complex_sample.gnucash",
                    env!("CARGO_MANIFEST_DIR")
                );

                tracing::info!("work_dir: {:?}", std::env::current_dir());
                let query = XMLQuery::new(path).unwrap();
                Book::new(query).await.unwrap()
            })
            .await
        }

        #[test(tokio::test)]
        async fn test_new() {
            let path: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );
            let query = XMLQuery::new(path).unwrap();
            Book::new(query).await.unwrap();
        }

        #[test(tokio::test)]
        async fn test_new_fail() {
            assert!(matches!(
                XMLQuery::new("tests/sample/no.gnucash"),
                Err(crate::Error::IO(_))
            ));
        }

        #[test(tokio::test)]
        async fn test_accounts() {
            let book = setup().await;
            let accounts = book.accounts().await.unwrap();
            // 少一組 Template Root
            assert_eq!(accounts.len(), 20);
        }

        #[test(tokio::test)]
        async fn test_accounts_contains_name() {
            let book = setup().await;
            let accounts = book.accounts_contains_name_ignore_case("aS").await.unwrap();
            assert_eq!(accounts.len(), 3);
        }

        #[test(tokio::test)]
        async fn test_account_contains_name() {
            let book = setup().await;
            let account = book
                .account_contains_name_ignore_case("NAS")
                .await
                .unwrap()
                .unwrap();
            assert_eq!(account.name, "NASDAQ");
        }

        #[test(tokio::test)]
        async fn test_splits() {
            let book = setup().await;
            let splits = book.splits().await.unwrap();
            assert_eq!(splits.len(), 25);
        }

        #[test(tokio::test)]
        async fn test_transactions() {
            let book = setup().await;
            let transactions = book.transactions().await.unwrap();
            assert_eq!(transactions.len(), 11);
        }

        #[test(tokio::test)]
        async fn test_prices() {
            let book = setup().await;
            let prices = book.prices().await.unwrap();
            assert_eq!(prices.len(), 5);
        }

        #[test(tokio::test)]
        async fn test_commodities() {
            let book = setup().await;
            let commodities = book.commodities().await.unwrap();
            // 多一組 template
            assert_eq!(commodities.len(), 6);
        }

        #[test(tokio::test)]
        async fn test_currencies() {
            let book = setup().await;
            let currencies = book.currencies().await.unwrap();
            assert_eq!(currencies.len(), 4);
        }

        #[test(tokio::test)]
        async fn test_exchange() {
            let book = setup().await;
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "ADF")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "AED")
                .unwrap();

            let rate = book.exchange(&commodity, &currency).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));
        }
    }
}
