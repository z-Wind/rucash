use chrono::NaiveDateTime;
use std::sync::Arc;

use crate::error::Error;
use crate::model::{Commodity, Split};
use crate::query::{CommodityQ, Query, SplitQ, TransactionT};

#[derive(Clone, Debug)]
pub struct Transaction<Q>
where
    Q: Query,
{
    query: Arc<Q>,

    pub guid: String,
    pub currency_guid: String,
    pub num: String,
    pub post_datetime: NaiveDateTime,
    pub enter_datetime: NaiveDateTime,
    pub description: String,
}

impl<Q> Transaction<Q>
where
    Q: Query,
{
    pub(crate) fn from_with_query<T: TransactionT>(item: &T, query: Arc<Q>) -> Self {
        Self {
            query,

            guid: item.guid(),
            currency_guid: item.currency_guid(),
            num: item.num(),
            post_datetime: item.post_datetime(),
            enter_datetime: item.enter_datetime(),
            description: item.description(),
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

    pub async fn splits(&self) -> Result<Vec<Split<Q>>, Error> {
        let splits = SplitQ::tx_guid(&*self.query, &self.guid).await?;
        Ok(splits
            .into_iter()
            .map(|x| Split::from_with_query(&x, self.query.clone()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Book;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::sqlite::transaction::Transaction as TransactionBase;
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
            let item = TransactionBase {
                guid: "guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                num: "currency_guid".to_string(),
                post_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok(),
                enter_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok(),
                description: Some("source".to_string()),
            };

            let result = Transaction::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.currency_guid, "currency_guid");
            assert_eq!(result.num, "currency_guid");
            assert_eq!(
                result.post_datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(
                result.enter_datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(result.description, "source");
        }

        #[tokio::test]
        async fn currency() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .unwrap();
            let currency = transaction.currency().await.unwrap();
            assert_eq!(currency.fullname, "Euro");
        }

        #[tokio::test]
        async fn splits() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .unwrap();
            let splits = transaction.splits().await.unwrap();
            assert_eq!(splits.len(), 2);
        }
    }

    #[cfg(feature = "sqlitefaster")]
    mod sqlitefaster {
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::sqlitefaster::transaction::Transaction as TransactionBase;
        use crate::SQLiteQueryFaster;

        #[allow(clippy::unused_async)]
        async fn setup() -> SQLiteQueryFaster {
            let uri: &str = &format!(
                "{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            println!("work_dir: {:?}", std::env::current_dir());
            SQLiteQueryFaster::new(uri).unwrap()
        }

        #[tokio::test]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = TransactionBase {
                guid: "guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                num: "currency_guid".to_string(),
                post_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok(),
                enter_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok(),
                description: Some("source".to_string()),
            };

            let result = Transaction::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.currency_guid, "currency_guid");
            assert_eq!(result.num, "currency_guid");
            assert_eq!(
                result.post_datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(
                result.enter_datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(result.description, "source");
        }

        #[tokio::test]
        async fn currency() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .unwrap();
            let currency = transaction.currency().await.unwrap();
            assert_eq!(currency.fullname, "Euro");
        }

        #[tokio::test]
        async fn splits() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .unwrap();
            let splits = transaction.splits().await.unwrap();
            assert_eq!(splits.len(), 2);
        }
    }

    #[cfg(feature = "mysql")]
    mod mysql {
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::mysql::transaction::Transaction as TransactionBase;
        use crate::MySQLQuery;

        async fn setup() -> MySQLQuery {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            MySQLQuery::new(uri).await.unwrap()
        }

        #[tokio::test]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = TransactionBase {
                guid: "guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                num: "currency_guid".to_string(),
                post_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok(),
                enter_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok(),
                description: Some("source".to_string()),
            };

            let result = Transaction::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.currency_guid, "currency_guid");
            assert_eq!(result.num, "currency_guid");
            assert_eq!(
                result.post_datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(
                result.enter_datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(result.description, "source");
        }

        #[tokio::test]
        async fn currency() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .unwrap();
            let currency = transaction.currency().await.unwrap();
            assert_eq!(currency.fullname, "Euro");
        }

        #[tokio::test]
        async fn splits() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .unwrap();
            let splits = transaction.splits().await.unwrap();
            assert_eq!(splits.len(), 2);
        }
    }

    #[cfg(feature = "postgresql")]
    mod postgresql {
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::postgresql::transaction::Transaction as TransactionBase;
        use crate::PostgreSQLQuery;

        async fn setup() -> PostgreSQLQuery {
            let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            PostgreSQLQuery::new(uri).await.unwrap()
        }

        #[tokio::test]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = TransactionBase {
                guid: "guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                num: "currency_guid".to_string(),
                post_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok(),
                enter_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok(),
                description: Some("source".to_string()),
            };

            let result = Transaction::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.currency_guid, "currency_guid");
            assert_eq!(result.num, "currency_guid");
            assert_eq!(
                result.post_datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(
                result.enter_datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(result.description, "source");
        }

        #[tokio::test]
        async fn currency() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .unwrap();
            let currency = transaction.currency().await.unwrap();
            assert_eq!(currency.fullname, "Euro");
        }

        #[tokio::test]
        async fn splits() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .unwrap();
            let splits = transaction.splits().await.unwrap();
            assert_eq!(splits.len(), 2);
        }
    }

    #[cfg(feature = "xml")]
    mod xml {
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::xml::transaction::Transaction as TransactionBase;
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
            let item = TransactionBase {
                guid: "guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                num: "currency_guid".to_string(),
                post_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok(),
                enter_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok(),
                description: Some("source".to_string()),
            };

            let result = Transaction::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.currency_guid, "currency_guid");
            assert_eq!(result.num, "currency_guid");
            assert_eq!(
                result.post_datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(
                result.enter_datetime,
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
            );
            assert_eq!(result.description, "source");
        }

        #[tokio::test]
        async fn currency() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .unwrap();
            let currency = transaction.currency().await.unwrap();
            assert_eq!(currency.fullname, "");
        }

        #[tokio::test]
        async fn splits() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let transaction = book
                .transactions()
                .await
                .unwrap()
                .into_iter()
                .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
                .unwrap();
            let splits = transaction.splits().await.unwrap();
            assert_eq!(splits.len(), 2);
        }
    }
}
