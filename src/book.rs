use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub async fn new(query: Q) -> Result<Self, Error> {
        let query = Arc::new(query);
        let mut book = Self {
            query,
            exchange_graph: None,
        };

        book.update_exchange_graph().await?;
        Ok(book)
    }

    pub async fn accounts(&self) -> Result<Vec<Account<Q>>, Error> {
        let accounts = self.query.accounts().await?;
        Ok(accounts
            .into_iter()
            .map(|x| Account::from_with_query(&x, self.query.clone()))
            .collect())
    }

    pub async fn accounts_contains_name_ignore_case(
        &self,
        name: &str,
    ) -> Result<Vec<Account<Q>>, Error> {
        let accounts = self.query.accounts_contains_name_ignore_case(name).await?;
        Ok(accounts
            .into_iter()
            .map(|x| Account::from_with_query(&x, self.query.clone()))
            .collect())
    }

    pub async fn account_contains_name_ignore_case(
        &self,
        name: &str,
    ) -> Result<Option<Account<Q>>, Error> {
        let mut accounts = self.accounts_contains_name_ignore_case(name).await?;
        match accounts.pop() {
            None => Ok(None),
            Some(x) if accounts.is_empty() => Ok(Some(x)),
            _ => Err(Error::NameMultipleFound {
                model: "Account".to_string(),
                name: name.to_string(),
            }),
        }
    }

    pub async fn splits(&self) -> Result<Vec<Split<Q>>, Error> {
        let splits = self.query.splits().await?;
        Ok(splits
            .into_iter()
            .map(|x| Split::from_with_query(&x, self.query.clone()))
            .collect())
    }

    pub async fn transactions(&self) -> Result<Vec<Transaction<Q>>, Error> {
        let transactions = self.query.transactions().await?;
        Ok(transactions
            .into_iter()
            .map(|x| Transaction::from_with_query(&x, self.query.clone()))
            .collect())
    }

    pub async fn prices(&self) -> Result<Vec<Price<Q>>, Error> {
        let prices = self.query.prices().await?;
        Ok(prices
            .into_iter()
            .map(|x| Price::from_with_query(&x, self.query.clone()))
            .collect())
    }

    pub async fn commodities(&self) -> Result<Vec<Commodity<Q>>, Error> {
        let commodities = self.query.commodities().await?;
        Ok(commodities
            .into_iter()
            .map(|x| Commodity::from_with_query(&x, self.query.clone()))
            .collect())
    }

    pub async fn currencies(&self) -> Result<Vec<Commodity<Q>>, Error> {
        let currencies = self.query.currencies().await?;
        Ok(currencies
            .into_iter()
            .map(|x| Commodity::from_with_query(&x, self.query.clone()))
            .collect())
    }

    pub async fn exchange(
        &self,
        commodity: &Commodity<Q>,
        currency: &Commodity<Q>,
    ) -> Option<crate::Num> {
        self.exchange_graph
            .as_ref()?
            .lock()
            .await
            .cal(commodity, currency)
    }

    pub async fn update_exchange_graph(&mut self) -> Result<(), Error> {
        match &self.exchange_graph {
            None => {
                let exchange_graph = Exchange::new(self.query.clone()).await?;
                self.exchange_graph = Some(Arc::new(Mutex::new(exchange_graph)));
                Ok(())
            }
            Some(graph) => graph.lock().await.update(self.query.clone()).await,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "sqlite")]
mod tests {
    use super::*;

    use crate::SQLiteQuery;

    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    use pretty_assertions::assert_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;
    use tokio::sync::OnceCell;

    static Q: OnceCell<Book<SQLiteQuery>> = OnceCell::const_new();
    async fn setup() -> &'static Book<SQLiteQuery> {
        Q.get_or_init(|| async {
            let uri: &str = &format!(
                "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            println!("work_dir: {:?}", std::env::current_dir());
            let query = SQLiteQuery::new(uri).await.unwrap();
            Book::new(query).await.unwrap()
        })
        .await
    }

    #[tokio::test]
    async fn test_new() {
        let uri: &str = &format!(
            "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
            env!("CARGO_MANIFEST_DIR")
        );
        let query = SQLiteQuery::new(uri).await.unwrap();
        Book::new(query).await.unwrap();
    }

    #[tokio::test]
    async fn test_new_fail() {
        assert!(matches!(
            SQLiteQuery::new("sqlite://tests/sample/no.gnucash").await,
            Err(crate::Error::Sql(_))
        ));
    }

    #[tokio::test]
    async fn test_accounts() {
        let book = setup().await;
        let accounts = book.accounts().await.unwrap();
        assert_eq!(accounts.len(), 21);
    }

    #[tokio::test]
    async fn test_accounts_contains_name() {
        let book = setup().await;
        let accounts = book.accounts_contains_name_ignore_case("aS").await.unwrap();
        assert_eq!(accounts.len(), 3);
    }

    #[tokio::test]
    async fn test_account_contains_name() {
        let book = setup().await;
        let account = book
            .account_contains_name_ignore_case("NAS")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(account.name, "NASDAQ");
    }

    #[tokio::test]
    async fn test_splits() {
        let book = setup().await;
        let splits = book.splits().await.unwrap();
        assert_eq!(splits.len(), 25);
    }

    #[tokio::test]
    async fn test_transactions() {
        let book = setup().await;
        let transactions = book.transactions().await.unwrap();
        assert_eq!(transactions.len(), 11);
    }

    #[tokio::test]
    async fn test_prices() {
        let book = setup().await;
        let prices = book.prices().await.unwrap();
        assert_eq!(prices.len(), 5);
    }

    #[tokio::test]
    async fn test_commodities() {
        let book = setup().await;
        let commodities = book.commodities().await.unwrap();
        assert_eq!(commodities.len(), 5);
    }

    #[tokio::test]
    async fn test_currencies() {
        let book = setup().await;
        let currencies = book.currencies().await.unwrap();
        assert_eq!(currencies.len(), 4);
    }

    #[tokio::test]
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
