// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use chrono::NaiveDateTime;
use rusqlite::Row;
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
    fn guid(&self) -> String {
        self.guid.clone()
    }
    fn currency_guid(&self) -> String {
        self.currency_guid.clone()
    }
    fn num(&self) -> String {
        self.num.clone()
    }
    fn post_datetime(&self) -> NaiveDateTime {
        self.post_date
    }
    fn enter_datetime(&self) -> NaiveDateTime {
        self.enter_date
    }
    fn description(&self) -> String {
        self.description.clone().unwrap_or_default()
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
    type T = Transaction;

    #[instrument(skip(self))]
    async fn all(&self) -> Result<Vec<Self::T>, Error> {
        tracing::debug!("fetching all transactions from sqlite");
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(SEL)
            .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;
        let result = stmt
            .query([])
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))?
            .mapped(|row| Transaction::try_from(row))
            .collect::<Result<Vec<_>, _>>()
            .inspect_err(|e| tracing::error!("failed to map query results: {e}"))?;
        tracing::debug!(count = result.len(), "transactions fetched from sqlite");
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn guid(&self, guid: &str) -> Result<Vec<Self::T>, Error> {
        tracing::debug!("fetching transaction by guid from sqlite");
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(&format!("{SEL}\nWHERE guid = ?"))
            .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;
        let result = stmt
            .query([guid])
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))?
            .mapped(|row| Transaction::try_from(row))
            .collect::<Result<Vec<_>, _>>()
            .inspect_err(|e| tracing::error!("failed to map query results: {e}"))?;
        tracing::debug!(count = result.len(), "transactions found by guid");
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn currency_guid(&self, guid: &str) -> Result<Vec<Self::T>, Error> {
        tracing::debug!("fetching transactions by currency_guid from sqlite");
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(&format!("{SEL}\nWHERE currency_guid = ?"))
            .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;
        let result = stmt
            .query([guid])
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))?
            .mapped(|row| Transaction::try_from(row))
            .collect::<Result<Vec<_>, _>>()
            .inspect_err(|e| tracing::error!("failed to map query results: {e}"))?;
        tracing::debug!(count = result.len(), "transactions found by currency_guid");
        Ok(result)
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
            .unwrap();

        let result = &result[0];
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
            .unwrap();

        assert_eq!(
            result[0].post_date,
            NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );

        assert_eq!(
            result[0].enter_date,
            NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S").unwrap()
        );
    }

    #[test(tokio::test)]
    async fn test_currency_guid() {
        let query = setup().await;
        let result = query
            .currency_guid("346629655191dcf59a7e2c2a85b70f69")
            .await
            .unwrap();

        assert_eq!(result.len(), 11);
    }
}
