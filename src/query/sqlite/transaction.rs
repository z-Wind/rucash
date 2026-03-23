// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use chrono::NaiveDateTime;
use rusqlite::Row;
use tokio::task::spawn_blocking;
use tracing::instrument;

use super::SQLiteQuery;
use crate::error::Error;
use crate::query::{TransactionQ, TransactionT};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Transaction {
    pub guid: String,
    pub currency_guid: String,
    pub num: String,
    pub post_date: NaiveDateTime,
    pub enter_date: NaiveDateTime,
    pub description: Option<String>,
}

impl<'a> TryFrom<&'a Row<'a>> for Transaction {
    type Error = rusqlite::Error;

    fn try_from(row: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            guid: row.get(0)?,
            currency_guid: row.get(1)?,
            num: row.get(2)?,
            post_date: row.get(3)?,
            enter_date: row.get(4)?,
            description: row.get(5)?,
        })
    }
}

impl TransactionT for Transaction {
    fn guid(&self) -> &str {
        &self.guid
    }
    fn currency_guid(&self) -> &str {
        &self.currency_guid
    }
    fn num(&self) -> &str {
        &self.num
    }
    fn post_datetime(&self) -> NaiveDateTime {
        self.post_date
    }
    fn enter_datetime(&self) -> NaiveDateTime {
        self.enter_date
    }
    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or_default()
    }
}

const SEL: &str = r"
SELECT
guid,
currency_guid,
num,
post_date,
enter_date,
description
FROM transactions
";

impl TransactionQ for SQLiteQuery {
    type Item = Transaction;

    #[instrument(skip(self))]
    async fn all(&self) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();

        spawn_blocking(move || {
            tracing::debug!("fetching all transactions from sqlite");

            let conn = pool.get()?;

            let mut stmt = conn
                .prepare_cached(SEL)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;

            let rows = stmt.query_map([], |row| Self::Item::try_from(row))?;

            let items = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(count = items.len(), "transactions fetched from sqlite");
            Ok(items)
        })
        .await
        .map_err(|e| Error::Internal(format!("Join error in spawn_blocking: {e}")))?
    }

    #[instrument(skip(self))]
    async fn guid(&self, guid: &str) -> Result<Option<Self::Item>, Error> {
        let pool = self.pool.clone();
        let guid_owned = guid.to_string();

        tokio::task::spawn_blocking(move || {
            tracing::debug!("fetching transaction by guid from sqlite");

            let conn = pool.get()?;

            let sql = format!("{SEL}\nWHERE guid = ?");
            let mut stmt = conn
                .prepare_cached(&sql)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;

            let result = stmt.query_row([guid_owned], |row| Self::Item::try_from(row));

            match result {
                Ok(item) => Ok(Some(item)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => {
                    tracing::error!("failed to fetch row: {e}");
                    Err(Error::from(e))
                }
            }
        })
        .await
        .map_err(|e| Error::Internal(format!("Join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn currency(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();
        let guid_owned = guid.to_string();

        tokio::task::spawn_blocking(move || {
            tracing::debug!("fetching transactions by currency_guid from sqlite");

            let conn = pool.get()?;

            let sql = format!("{SEL}\nWHERE currency_guid = ?");
            let mut stmt = conn
                .prepare_cached(&sql)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;

            let rows = stmt.query_map([guid_owned], |row| Self::Item::try_from(row))?;

            let items = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(count = items.len(), "transactions found by currency_guid");
            Ok(items)
        })
        .await
        .map_err(|e| Error::Internal(format!("Join error: {e}")))?
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_log::test;
    use tokio::sync::OnceCell;

    use super::*;

    #[cfg(feature = "schema")]
    // test schemas on compile time
    #[allow(dead_code)]
    fn test_transaction_schemas() {
        let _ = sqlx::query_as!(
            Transaction,
            r#"
				SELECT
				guid,
				currency_guid,
				num,
				post_date as "post_date!: NaiveDateTime",
				enter_date as "enter_date!: NaiveDateTime",
				description
				FROM transactions
				"#,
        );
    }

    static Q: OnceCell<SQLiteQuery> = OnceCell::const_new();
    async fn setup() -> &'static SQLiteQuery {
        Q.get_or_init(|| async {
            let uri: &str = &format!(
                "{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            tracing::debug!("work_dir: {:?}", std::env::current_dir());
            SQLiteQuery::new(uri).unwrap()
        })
        .await
    }

    #[test(tokio::test)]
    async fn test_transaction() {
        let query = setup().await;
        let result = query
            .guid("6c8876003c4a6026e38e3afb67d6f2b1")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result.guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
        assert_eq!(result.currency_guid(), "346629655191dcf59a7e2c2a85b70f69");
        assert_eq!(result.num(), "");
        assert_eq!(
            result.post_datetime(),
            NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(
            result.enter_datetime(),
            NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(result.description(), "income 1");
    }

    #[test(tokio::test)]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 11);
    }

    #[test(tokio::test)]
    async fn test_by_guid() {
        let query = setup().await;
        let result = query
            .guid("6c8876003c4a6026e38e3afb67d6f2b1")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            result.post_date,
            NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );

        assert_eq!(
            result.enter_date,
            NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S").unwrap()
        );
    }

    #[test(tokio::test)]
    async fn test_currency_guid() {
        let query = setup().await;
        let result = query
            .currency("346629655191dcf59a7e2c2a85b70f69")
            .await
            .unwrap();

        assert_eq!(result.len(), 11);
    }
}
