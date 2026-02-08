use chrono::NaiveDateTime;
use std::sync::Arc;
use tracing::instrument;

use crate::error::Error;
use crate::model::Commodity;
use crate::query::{CommodityQ, PriceT, Query};

#[derive(Clone, Debug)]
pub struct Price<Q>
where
    Q: Query,
{
    query: Arc<Q>,

    pub guid: String,
    pub commodity_guid: String,
    pub currency_guid: String,
    pub datetime: NaiveDateTime,
    pub source: String,
    pub r#type: String,
    pub value: crate::Num,
}

impl<Q> Price<Q>
where
    Q: Query,
{
    pub(crate) fn from_with_query<T: PriceT>(item: &T, query: Arc<Q>) -> Self {
        Self {
            query,

            guid: item.guid(),
            commodity_guid: item.commodity_guid(),
            currency_guid: item.currency_guid(),
            datetime: item.datetime(),
            source: item.source(),
            r#type: item.r#type(),
            value: item.value(),
        }
    }

    #[instrument(skip(self), fields(price_guid = %self.guid, commodity_guid = %self.commodity_guid))]
    pub async fn commodity(&self) -> Result<Commodity<Q>, Error> {
        if self.commodity_guid.is_empty() {
            tracing::error!("commodity guid is empty");
            return Err(Error::GuidNotFound {
                model: "Commodity".to_string(),
                guid: self.commodity_guid.clone(),
            });
        }

        tracing::debug!("fetching commodity for price");
        let mut commodities = CommodityQ::guid(&*self.query, &self.commodity_guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch commodity: {e}"))?;
        match commodities.pop() {
            None => {
                tracing::error!("commodity not found");
                Err(Error::GuidNotFound {
                    model: "Commodity".to_string(),
                    guid: self.commodity_guid.clone(),
                })
            }
            Some(x) if commodities.is_empty() => {
                tracing::debug!("commodity found for price");
                Ok(Commodity::from_with_query(&x, self.query.clone()))
            }
            _ => {
                tracing::error!("multiple commodities found for guid");
                Err(Error::GuidMultipleFound {
                    model: "Commodity".to_string(),
                    guid: self.commodity_guid.clone(),
                })
            }
        }
    }

    #[instrument(skip(self), fields(price_guid = %self.guid, currency_guid = %self.currency_guid))]
    pub async fn currency(&self) -> Result<Commodity<Q>, Error> {
        if self.currency_guid.is_empty() {
            tracing::error!("currency guid is empty");
            return Err(Error::GuidNotFound {
                model: "Commodity".to_string(),
                guid: self.currency_guid.clone(),
            });
        }

        tracing::debug!("fetching currency for price");
        let mut currencies = CommodityQ::guid(&*self.query, &self.currency_guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch currency: {e}"))?;

        match currencies.pop() {
            None => {
                tracing::error!("currency not found");
                Err(Error::GuidNotFound {
                    model: "Commodity".to_string(),
                    guid: self.currency_guid.clone(),
                })
            }
            Some(x) if currencies.is_empty() => {
                tracing::debug!("currency found for price");
                Ok(Commodity::from_with_query(&x, self.query.clone()))
            }
            _ => {
                tracing::error!("multiple currencies found for guid");
                Err(Error::GuidMultipleFound {
                    model: "Commodity".to_string(),
                    guid: self.currency_guid.clone(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Book;

    use super::*;

    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::SQLiteQuery;
        use crate::query::sqlite::price::Price as PriceBase;

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
            let item = PriceBase {
                guid: "guid".to_string(),
                commodity_guid: "commodity_guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                date: NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
                source: Some("source".to_string()),
                r#type: Some("type".to_string()),
                value_num: 1000,
                value_denom: 10,
            };

            let result = Price::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.commodity_guid, "commodity_guid");
            assert_eq!(result.currency_guid, "currency_guid");
            assert_eq!(
                result.datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(result.source, "source");
            assert_eq!(result.r#type, "type");
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, result.value, 100.0);
            #[cfg(feature = "decimal")]
            assert_eq!(result.value, Decimal::new(100, 0));
        }

        #[test(tokio::test)]
        async fn commodity() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();
            let commodity = price.commodity().await.unwrap();
            assert_eq!(commodity.fullname, "Andorran Franc");
        }

        #[test(tokio::test)]
        async fn currency() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();
            let currency = price.currency().await.unwrap();
            assert_eq!(currency.fullname, "UAE Dirham");
        }
    }

    #[cfg(feature = "mysql")]
    mod mysql {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::MySQLQuery;
        use crate::query::mysql::price::Price as PriceBase;

        use super::*;

        async fn setup() -> MySQLQuery {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            MySQLQuery::new(uri).await.unwrap()
        }

        #[test(tokio::test)]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = PriceBase {
                guid: "guid".to_string(),
                commodity_guid: "commodity_guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                date: NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
                source: Some("source".to_string()),
                r#type: Some("type".to_string()),
                value_num: 1000,
                value_denom: 10,
            };

            let result = Price::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.commodity_guid, "commodity_guid");
            assert_eq!(result.currency_guid, "currency_guid");
            assert_eq!(
                result.datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(result.source, "source");
            assert_eq!(result.r#type, "type");
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, result.value, 100.0);
            #[cfg(feature = "decimal")]
            assert_eq!(result.value, Decimal::new(100, 0));
        }

        #[test(tokio::test)]
        async fn commodity() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();
            let commodity = price.commodity().await.unwrap();
            assert_eq!(commodity.fullname, "Andorran Franc");
        }

        #[test(tokio::test)]
        async fn currency() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();
            let currency = price.currency().await.unwrap();
            assert_eq!(currency.fullname, "UAE Dirham");
        }
    }

    #[cfg(feature = "postgresql")]
    mod postgresql {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::PostgreSQLQuery;
        use crate::query::postgresql::price::Price as PriceBase;

        use super::*;

        async fn setup() -> PostgreSQLQuery {
            let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            PostgreSQLQuery::new(uri).await.unwrap()
        }

        #[test(tokio::test)]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = PriceBase {
                guid: "guid".to_string(),
                commodity_guid: "commodity_guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                date: NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
                source: Some("source".to_string()),
                r#type: Some("type".to_string()),
                value_num: 1000,
                value_denom: 10,
            };

            let result = Price::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.commodity_guid, "commodity_guid");
            assert_eq!(result.currency_guid, "currency_guid");
            assert_eq!(
                result.datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(result.source, "source");
            assert_eq!(result.r#type, "type");
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, result.value, 100.0);
            #[cfg(feature = "decimal")]
            assert_eq!(result.value, Decimal::new(100, 0));
        }

        #[test(tokio::test)]
        async fn commodity() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();
            let commodity = price.commodity().await.unwrap();
            assert_eq!(commodity.fullname, "Andorran Franc");
        }

        #[test(tokio::test)]
        async fn currency() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();
            let currency = price.currency().await.unwrap();
            assert_eq!(currency.fullname, "UAE Dirham");
        }
    }

    #[cfg(feature = "xml")]
    mod xml {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::XMLQuery;
        use crate::query::xml::price::Price as PriceBase;

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
            let item = PriceBase {
                guid: "guid".to_string(),
                commodity_guid: "commodity_guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                date: NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
                source: Some("source".to_string()),
                r#type: Some("type".to_string()),
                value_num: 1000,
                value_denom: 10,
            };

            let result = Price::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.commodity_guid, "commodity_guid");
            assert_eq!(result.currency_guid, "currency_guid");
            assert_eq!(
                result.datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(result.source, "source");
            assert_eq!(result.r#type, "type");
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, result.value, 100.0);
            #[cfg(feature = "decimal")]
            assert_eq!(result.value, Decimal::new(100, 0));
        }

        #[test(tokio::test)]
        async fn commodity() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();
            let commodity = price.commodity().await.unwrap();
            assert_eq!(commodity.fullname, "");
        }

        #[test(tokio::test)]
        async fn currency() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let price = book
                .prices()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();
            let currency = price.currency().await.unwrap();
            assert_eq!(currency.fullname, "");
        }
    }
}
