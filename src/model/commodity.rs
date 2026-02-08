use std::sync::Arc;

use tracing::instrument;

use crate::book::Book;
use crate::error::Error;
use crate::model::{Account, Price, Transaction};
use crate::query::{AccountQ, CommodityT, PriceQ, Query, TransactionQ};

#[derive(Clone, Debug)]
pub struct Commodity<Q>
where
    Q: Query,
{
    query: Arc<Q>,

    pub guid: String,
    pub namespace: String,
    pub mnemonic: String,
    pub fullname: String,
    pub cusip: String,
    pub fraction: i64,
    pub quote_flag: bool,
    pub quote_source: String,
    pub quote_tz: String,
}

impl<Q> Commodity<Q>
where
    Q: Query,
{
    pub(crate) fn from_with_query<T: CommodityT>(item: &T, query: Arc<Q>) -> Self {
        Self {
            query,

            guid: item.guid(),
            namespace: item.namespace(),
            mnemonic: item.mnemonic(),
            fullname: item.fullname(),
            cusip: item.cusip(),
            fraction: item.fraction(),
            quote_flag: item.quote_flag(),
            quote_source: item.quote_source(),
            quote_tz: item.quote_tz(),
        }
    }

    #[instrument(skip(self), fields(commodity_guid = %self.guid, commodity_mnemonic = %self.mnemonic))]
    pub async fn accounts(&self) -> Result<Vec<Account<Q>>, Error> {
        tracing::debug!("fetching accounts for commodity");
        let accounts = AccountQ::commodity_guid(&*self.query, &self.guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch accounts: {e}"))?;
        let result: Vec<_> = accounts
            .into_iter()
            .map(|x| Account::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::debug!(count = result.len(), "accounts fetched for commodity");
        Ok(result)
    }

    #[instrument(skip(self), fields(commodity_guid = %self.guid, commodity_mnemonic = %self.mnemonic))]
    pub async fn transactions(&self) -> Result<Vec<Transaction<Q>>, Error> {
        tracing::debug!("fetching transactions for commodity");
        let transactions = TransactionQ::currency_guid(&*self.query, &self.guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch transactions: {e}"))?;
        let result: Vec<_> = transactions
            .into_iter()
            .map(|x| Transaction::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::debug!(count = result.len(), "transactions fetched for commodity");
        Ok(result)
    }

    #[instrument(skip(self), fields(commodity_guid = %self.guid))]
    pub async fn as_commodity_prices(&self) -> Result<Vec<Price<Q>>, Error> {
        tracing::debug!("fetching prices where this is the commodity");
        let prices = PriceQ::commodity_guid(&*self.query, &self.guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch prices: {e}"))?;
        let result: Vec<_> = prices
            .into_iter()
            .map(|x| Price::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::debug!(count = result.len(), "commodity prices fetched");
        Ok(result)
    }

    #[instrument(skip(self), fields(commodity_guid = %self.guid))]
    pub async fn as_currency_prices(&self) -> Result<Vec<Price<Q>>, Error> {
        tracing::debug!("fetching prices where this is the currency");
        let prices = PriceQ::currency_guid(&*self.query, &self.guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch prices: {e}"))?;
        let result: Vec<_> = prices
            .into_iter()
            .map(|x| Price::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::debug!(count = result.len(), "currency prices fetched");
        Ok(result)
    }

    #[instrument(skip(self), fields(commodity_guid = %self.guid))]
    pub async fn as_commodity_or_currency_prices(&self) -> Result<Vec<Price<Q>>, Error> {
        tracing::debug!("fetching prices where this is commodity or currency");
        let prices = PriceQ::commodity_or_currency_guid(&*self.query, &self.guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch prices: {e}"))?;
        let result: Vec<_> = prices
            .into_iter()
            .map(|x| Price::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::debug!(count = result.len(), "all related prices fetched");
        Ok(result)
    }

    #[instrument(skip(self, currency, book), fields(
        from_mnemonic = %self.mnemonic,
        to_mnemonic = %currency.mnemonic
    ))]
    pub async fn sell(&self, currency: &Self, book: &Book<Q>) -> Option<crate::Num> {
        tracing::debug!("selling commodity to currency");
        let result = book.exchange(self, currency).await;
        if result.is_some() {
            tracing::debug!(?result, "sell exchange rate found");
        } else {
            tracing::warn!("no sell exchange rate available");
        }
        result
    }

    #[instrument(skip(self, commodity, book), fields(
        buying_mnemonic = %commodity.mnemonic,
        with_mnemonic = %self.mnemonic
    ))]
    pub async fn buy(&self, commodity: &Self, book: &Book<Q>) -> Option<crate::Num> {
        tracing::debug!("buying commodity with currency");
        let result = commodity.sell(self, book).await;
        if result.is_some() {
            tracing::debug!(?result, "buy exchange rate found");
        } else {
            tracing::warn!("no buy exchange rate available");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    use super::*;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::SQLiteQuery;
        use crate::query::sqlite::commodity::Commodity as CommodityBase;

        use super::*;

        #[allow(clippy::unused_async)]
        async fn setup() -> SQLiteQuery {
            let uri: &str = &format!(
                "{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            tracing::info!("work_dir: {:?}", std::env::current_dir());
            SQLiteQuery::new(uri).unwrap()
        }

        #[test(tokio::test)]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = CommodityBase {
                guid: "guid".to_string(),
                namespace: "namespace".to_string(),
                mnemonic: "mnemonic".to_string(),
                fullname: Some("fullname".to_string()),
                cusip: Some("cusip".to_string()),
                fraction: 100,
                quote_flag: 1,
                quote_source: Some("quote_source".to_string()),
                quote_tz: Some("quote_tz".to_string()),
            };

            let result = Commodity::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.namespace, "namespace");
            assert_eq!(result.mnemonic, "mnemonic");
            assert_eq!(result.fullname, "fullname");
            assert_eq!(result.cusip, "cusip");
            assert_eq!(result.fraction, 100);
            assert_eq!(result.quote_flag, true);
            assert_eq!(result.quote_source, "quote_source");
            assert_eq!(result.quote_tz, "quote_tz");
        }

        #[test(tokio::test)]
        async fn test_accounts() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let accounts = commodity.accounts().await.unwrap();
            assert_eq!(accounts.len(), 14);
        }

        #[test(tokio::test)]
        async fn test_transactions() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let transactions = commodity.transactions().await.unwrap();
            assert_eq!(transactions.len(), 11);
        }

        #[test(tokio::test)]
        async fn test_as_commodity_prices() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let prices = commodity.as_commodity_prices().await.unwrap();
            assert_eq!(prices.len(), 1);
        }

        #[test(tokio::test)]
        async fn test_as_currency_prices() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let prices = commodity.as_currency_prices().await.unwrap();
            assert_eq!(prices.len(), 2);
        }

        #[test(tokio::test)]
        async fn test_as_commodity_or_currency_prices() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let prices = commodity.as_commodity_or_currency_prices().await.unwrap();
            assert_eq!(prices.len(), 3);
        }

        #[test(tokio::test)]
        async fn test_rate_direct() {
            // ADF => AED
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
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

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));

            let rate = currency.buy(&commodity, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));

            // AED => EUR
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 9.0 / 10.0);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));

            let rate = currency.buy(&commodity, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 9.0 / 10.0);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));
        }

        #[test(tokio::test)]
        async fn test_rate_indirect() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            // USD => AED
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "1e5d65e2726a5d4595741cb204992991")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .unwrap();

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 7.0 / 5.0 * 10.0 / 9.0);
            #[cfg(feature = "decimal")]
            assert_eq!(
                rate,
                (Decimal::new(7, 0) / Decimal::new(5, 0))
                    * (Decimal::new(10, 0) / Decimal::new(9, 0)),
            );
        }
    }

    #[cfg(feature = "mysql")]
    mod mysql {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::MySQLQuery;
        use crate::query::mysql::commodity::Commodity as CommodityBase;

        use super::*;

        async fn setup() -> MySQLQuery {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            MySQLQuery::new(uri).await.unwrap()
        }

        #[test(tokio::test)]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = CommodityBase {
                guid: "guid".to_string(),
                namespace: "namespace".to_string(),
                mnemonic: "mnemonic".to_string(),
                fullname: Some("fullname".to_string()),
                cusip: Some("cusip".to_string()),
                fraction: 100,
                quote_flag: 1,
                quote_source: Some("quote_source".to_string()),
                quote_tz: Some("quote_tz".to_string()),
            };

            let result = Commodity::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.namespace, "namespace");
            assert_eq!(result.mnemonic, "mnemonic");
            assert_eq!(result.fullname, "fullname");
            assert_eq!(result.cusip, "cusip");
            assert_eq!(result.fraction, 100);
            assert_eq!(result.quote_flag, true);
            assert_eq!(result.quote_source, "quote_source");
            assert_eq!(result.quote_tz, "quote_tz");
        }

        #[test(tokio::test)]
        async fn test_accounts() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let accounts = commodity.accounts().await.unwrap();
            assert_eq!(accounts.len(), 14);
        }

        #[test(tokio::test)]
        async fn test_transactions() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let transactions = commodity.transactions().await.unwrap();
            assert_eq!(transactions.len(), 11);
        }

        #[test(tokio::test)]
        async fn test_as_commodity_prices() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let prices = commodity.as_commodity_prices().await.unwrap();
            assert_eq!(prices.len(), 1);
        }

        #[test(tokio::test)]
        async fn test_as_currency_prices() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let prices = commodity.as_currency_prices().await.unwrap();
            assert_eq!(prices.len(), 2);
        }

        #[test(tokio::test)]
        async fn test_as_commodity_or_currency_prices() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let prices = commodity.as_commodity_or_currency_prices().await.unwrap();
            assert_eq!(prices.len(), 3);
        }

        #[test(tokio::test)]
        async fn test_rate_direct() {
            // ADF => AED
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
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

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));

            let rate = currency.buy(&commodity, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));

            // AED => EUR
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 9.0 / 10.0);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));

            let rate = currency.buy(&commodity, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 9.0 / 10.0);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));
        }

        #[test(tokio::test)]
        async fn test_rate_indirect() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            // USD => AED
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "1e5d65e2726a5d4595741cb204992991")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .unwrap();

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 7.0 / 5.0 * 10.0 / 9.0);
            #[cfg(feature = "decimal")]
            assert_eq!(
                rate,
                (Decimal::new(7, 0) / Decimal::new(5, 0))
                    * (Decimal::new(10, 0) / Decimal::new(9, 0)),
            );
        }
    }

    #[cfg(feature = "postgresql")]
    mod postgresql {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::PostgreSQLQuery;
        use crate::query::postgresql::commodity::Commodity as CommodityBase;

        use super::*;

        async fn setup() -> PostgreSQLQuery {
            let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            PostgreSQLQuery::new(uri).await.unwrap()
        }

        #[test(tokio::test)]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = CommodityBase {
                guid: "guid".to_string(),
                namespace: "namespace".to_string(),
                mnemonic: "mnemonic".to_string(),
                fullname: Some("fullname".to_string()),
                cusip: Some("cusip".to_string()),
                fraction: 100,
                quote_flag: 1,
                quote_source: Some("quote_source".to_string()),
                quote_tz: Some("quote_tz".to_string()),
            };

            let result = Commodity::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.namespace, "namespace");
            assert_eq!(result.mnemonic, "mnemonic");
            assert_eq!(result.fullname, "fullname");
            assert_eq!(result.cusip, "cusip");
            assert_eq!(result.fraction, 100);
            assert_eq!(result.quote_flag, true);
            assert_eq!(result.quote_source, "quote_source");
            assert_eq!(result.quote_tz, "quote_tz");
        }

        #[test(tokio::test)]
        async fn test_accounts() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let accounts = commodity.accounts().await.unwrap();
            assert_eq!(accounts.len(), 14);
        }

        #[test(tokio::test)]
        async fn test_transactions() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let transactions = commodity.transactions().await.unwrap();
            assert_eq!(transactions.len(), 11);
        }

        #[test(tokio::test)]
        async fn test_as_commodity_prices() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let prices = commodity.as_commodity_prices().await.unwrap();
            assert_eq!(prices.len(), 1);
        }

        #[test(tokio::test)]
        async fn test_as_currency_prices() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let prices = commodity.as_currency_prices().await.unwrap();
            assert_eq!(prices.len(), 2);
        }

        #[test(tokio::test)]
        async fn test_as_commodity_or_currency_prices() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();
            let prices = commodity.as_commodity_or_currency_prices().await.unwrap();
            assert_eq!(prices.len(), 3);
        }

        #[test(tokio::test)]
        async fn test_rate_direct() {
            // ADF => AED
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
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

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));

            let rate = currency.buy(&commodity, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));

            // AED => EUR
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
                .unwrap();

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 9.0 / 10.0);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));

            let rate = currency.buy(&commodity, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 9.0 / 10.0);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));
        }

        #[test(tokio::test)]
        async fn test_rate_indirect() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            // USD => AED
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "1e5d65e2726a5d4595741cb204992991")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
                .unwrap();

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 7.0 / 5.0 * 10.0 / 9.0);
            #[cfg(feature = "decimal")]
            assert_eq!(
                rate,
                (Decimal::new(7, 0) / Decimal::new(5, 0))
                    * (Decimal::new(10, 0) / Decimal::new(9, 0)),
            );
        }
    }

    #[cfg(feature = "xml")]
    mod xml {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::XMLQuery;
        use crate::query::xml::commodity::Commodity as CommodityBase;

        use super::*;

        fn setup() -> XMLQuery {
            let path: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            tracing::info!("work_dir: {:?}", std::env::current_dir());
            XMLQuery::new(path).unwrap()
        }

        #[test(tokio::test)]
        async fn test_from_with_query() {
            let query = Arc::new(setup());
            let item = CommodityBase {
                guid: "guid".to_string(),
                namespace: "namespace".to_string(),
                mnemonic: "mnemonic".to_string(),
                fullname: Some("fullname".to_string()),
                cusip: Some("cusip".to_string()),
                fraction: 100,
                quote_flag: true,
                quote_source: Some("quote_source".to_string()),
                quote_tz: Some("quote_tz".to_string()),
            };

            let result = Commodity::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.namespace, "namespace");
            assert_eq!(result.mnemonic, "mnemonic");
            assert_eq!(result.fullname, "fullname");
            assert_eq!(result.cusip, "cusip");
            assert_eq!(result.fraction, 100);
            assert_eq!(result.quote_flag, true);
            assert_eq!(result.quote_source, "quote_source");
            assert_eq!(result.quote_tz, "quote_tz");
        }

        #[test(tokio::test)]
        async fn test_accounts() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "EUR")
                .unwrap();
            let accounts = commodity.accounts().await.unwrap();
            assert_eq!(accounts.len(), 14);
        }

        #[test(tokio::test)]
        async fn test_transactions() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "EUR")
                .unwrap();
            let transactions = commodity.transactions().await.unwrap();
            assert_eq!(transactions.len(), 11);
        }

        #[test(tokio::test)]
        async fn test_as_commodity_prices() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "EUR")
                .unwrap();
            let prices = commodity.as_commodity_prices().await.unwrap();
            assert_eq!(prices.len(), 1);
        }

        #[test(tokio::test)]
        async fn test_as_currency_prices() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "EUR")
                .unwrap();
            let prices = commodity.as_currency_prices().await.unwrap();
            assert_eq!(prices.len(), 2);
        }

        #[test(tokio::test)]
        async fn test_as_commodity_or_currency_prices() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "EUR")
                .unwrap();
            let prices = commodity.as_commodity_or_currency_prices().await.unwrap();
            assert_eq!(prices.len(), 3);
        }

        #[test(tokio::test)]
        async fn test_rate_direct() {
            // ADF => AED
            let query = setup();
            let book = Book::new(query).await.unwrap();
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

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));

            let rate = currency.buy(&commodity, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 1.5);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(15, 1));

            // AED => EUR
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "AED")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "EUR")
                .unwrap();

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 9.0 / 10.0);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));

            let rate = currency.buy(&commodity, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 9.0 / 10.0);
            #[cfg(feature = "decimal")]
            assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));
        }

        #[test(tokio::test)]
        async fn test_rate_indirect() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            // USD => AED
            let commodity = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "USD")
                .unwrap();
            let currency = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "AED")
                .unwrap();

            let rate = commodity.sell(&currency, &book).await.unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, rate, 7.0 / 5.0 * 10.0 / 9.0);
            #[cfg(feature = "decimal")]
            assert_eq!(
                rate,
                (Decimal::new(7, 0) / Decimal::new(5, 0))
                    * (Decimal::new(10, 0) / Decimal::new(9, 0)),
            );
        }
    }
}
