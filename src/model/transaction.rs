use chrono::NaiveDateTime;
use std::sync::Arc;
use tracing::instrument;

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

    #[instrument(skip(self), fields(transaction_guid = %self.guid, currency_guid = %self.currency_guid))]
    pub async fn currency(&self) -> Result<Commodity<Q>, Error> {
        if self.currency_guid.is_empty() {
            tracing::error!("currency guid is empty");
            return Err(Error::GuidNotFound {
                model: "Commodity".to_string(),
                guid: self.currency_guid.clone(),
            });
        }

        tracing::debug!("fetching currency for transaction");
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
                tracing::info!("currency found for transaction");
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

    #[instrument(skip(self), fields(transaction_guid = %self.guid))]
    pub async fn splits(&self) -> Result<Vec<Split<Q>>, Error> {
        tracing::debug!("fetching splits for transaction");
        let splits = SplitQ::tx_guid(&*self.query, &self.guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch splits: {e}"))?;
        let result: Vec<_> = splits
            .into_iter()
            .map(|x| Split::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::info!(count = result.len(), "splits fetched for transaction");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::Book;

    use super::*;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::SQLiteQuery;
        use crate::query::sqlite::transaction::Transaction as TransactionBase;

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
            let item = TransactionBase {
                guid: "guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                num: "currency_guid".to_string(),
                post_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
                enter_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::MySQLQuery;
        use crate::query::mysql::transaction::Transaction as TransactionBase;

        use super::*;

        async fn setup() -> MySQLQuery {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            MySQLQuery::new(uri).await.unwrap()
        }

        #[test(tokio::test)]
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
                .unwrap(),
                enter_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::PostgreSQLQuery;
        use crate::query::postgresql::transaction::Transaction as TransactionBase;

        use super::*;

        async fn setup() -> PostgreSQLQuery {
            let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            PostgreSQLQuery::new(uri).await.unwrap()
        }

        #[test(tokio::test)]
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
                .unwrap(),
                enter_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::XMLQuery;
        use crate::query::xml::transaction::Transaction as TransactionBase;

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
            let item = TransactionBase {
                guid: "guid".to_string(),
                currency_guid: "currency_guid".to_string(),
                num: "currency_guid".to_string(),
                post_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
                enter_date: NaiveDateTime::parse_from_str(
                    "2014-12-24 10:59:00",
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
