use chrono::NaiveDateTime;
use std::sync::Arc;

use crate::error::Error;
use crate::model::Commodity;
use crate::query::{CommodityQ, PriceT, Query};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "decimal", derive(Eq, Hash))]
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

    pub async fn commodity(&self) -> Result<Commodity<Q>, Error> {
        if self.commodity_guid.is_empty() {
            return Err(Error::GuidNotFound {
                model: "Commodity".to_string(),
                guid: self.commodity_guid.clone(),
            });
        }

        let mut commodities = CommodityQ::guid(&*self.query, &self.commodity_guid).await?;
        match commodities.pop() {
            None => Err(Error::GuidNotFound {
                model: "Commodity".to_string(),
                guid: self.commodity_guid.clone(),
            }),
            Some(x) if commodities.is_empty() => {
                Ok(Commodity::from_with_query(&x, self.query.clone()))
            }
            _ => Err(Error::GuidMultipleFound {
                model: "Commodity".to_string(),
                guid: self.commodity_guid.clone(),
            }),
        }
    }

    pub async fn currency(&self) -> Result<Commodity<Q>, Error> {
        if self.currency_guid.is_empty() {
            return Err(Error::GuidNotFound {
                model: "Commodity".to_string(),
                guid: self.currency_guid.clone(),
            });
        };

        let mut currencies = CommodityQ::guid(&*self.query, &self.currency_guid).await?;

        match currencies.pop() {
            None => Err(Error::GuidNotFound {
                model: "Commodity".to_string(),
                guid: self.currency_guid.clone(),
            }),
            Some(x) if currencies.is_empty() => {
                Ok(Commodity::from_with_query(&x, self.query.clone()))
            }
            _ => Err(Error::GuidMultipleFound {
                model: "Commodity".to_string(),
                guid: self.currency_guid.clone(),
            }),
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    use crate::Book;

    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::sqlite::price::Price as PriceBase;
        use crate::SQLiteQuery;

        async fn setup() -> SQLiteQuery {
            let uri: &str = &format!(
                "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            println!("work_dir: {:?}", std::env::current_dir());
            SQLiteQuery::new(&format!("{uri}?mode=ro")).await.unwrap()
        }

        #[tokio::test]
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

        #[tokio::test]
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

        #[tokio::test]
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
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::mysql::price::Price as PriceBase;
        use crate::MySQLQuery;

        async fn setup() -> MySQLQuery {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            MySQLQuery::new(uri).await.unwrap()
        }

        #[tokio::test]
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

        #[tokio::test]
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

        #[tokio::test]
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
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::postgresql::price::Price as PriceBase;
        use crate::PostgreSQLQuery;

        async fn setup() -> PostgreSQLQuery {
            let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            PostgreSQLQuery::new(uri).await.unwrap()
        }

        #[tokio::test]
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

        #[tokio::test]
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

        #[tokio::test]
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
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::xml::price::Price as PriceBase;
        use crate::XMLQuery;

        fn setup() -> XMLQuery {
            let path: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            println!("work_dir: {:?}", std::env::current_dir());
            XMLQuery::new(path).unwrap()
        }

        #[tokio::test]
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

        #[tokio::test]
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

        #[tokio::test]
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
