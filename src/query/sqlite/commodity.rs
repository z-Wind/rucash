// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL
use rusqlite::Row;
use tokio::task::spawn_blocking;
use tracing::instrument;

use super::SQLiteQuery;
use crate::error::Error;
use crate::query::{CommodityQ, CommodityT};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Commodity {
    pub(crate) guid: String,
    pub(crate) namespace: String,
    pub(crate) mnemonic: String,
    pub(crate) fullname: Option<String>,
    pub(crate) cusip: Option<String>,
    pub(crate) fraction: i64,
    pub(crate) quote_flag: i64,
    pub(crate) quote_source: Option<String>,
    pub(crate) quote_tz: Option<String>,
}

impl<'a> TryFrom<&'a Row<'a>> for Commodity {
    type Error = rusqlite::Error;

    fn try_from(row: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            guid: row.get(0)?,
            namespace: row.get(1)?,
            mnemonic: row.get(2)?,
            fullname: row.get(3)?,
            cusip: row.get(4)?,
            fraction: row.get(5)?,
            quote_flag: row.get(6)?,
            quote_source: row.get(7)?,
            quote_tz: row.get(8)?,
        })
    }
}

impl CommodityT for Commodity {
    fn guid(&self) -> &str {
        &self.guid
    }
    fn namespace(&self) -> &str {
        &self.namespace
    }
    fn mnemonic(&self) -> &str {
        &self.mnemonic
    }
    fn fullname(&self) -> &str {
        self.fullname.as_deref().unwrap_or_default()
    }
    fn cusip(&self) -> &str {
        self.cusip.as_deref().unwrap_or_default()
    }
    fn fraction(&self) -> i64 {
        self.fraction
    }
    fn quote_flag(&self) -> bool {
        self.quote_flag != 0
    }
    fn quote_source(&self) -> &str {
        self.quote_source.as_deref().unwrap_or_default()
    }
    fn quote_tz(&self) -> &str {
        self.quote_tz.as_deref().unwrap_or_default()
    }
}

const SEL: &str = r"
SELECT
guid,
namespace,
mnemonic,
fullname,
cusip,
fraction,
quote_flag,
quote_source,
quote_tz
FROM commodities
";

impl CommodityQ for SQLiteQuery {
    type Item = Commodity;

    #[instrument(skip(self))]
    async fn all(&self) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();

        spawn_blocking(move || {
            tracing::debug!("fetching all commodities from sqlite");

            let conn = pool.get()?;

            let mut stmt = conn
                .prepare_cached(SEL)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;

            let rows = stmt.query_map([], |row| Self::Item::try_from(row))?;

            let items = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(count = items.len(), "commodities fetched from sqlite");
            Ok(items)
        })
        .await
        .map_err(|e| Error::Internal(format!("Join error in spawn_blocking: {e}")))?
    }

    #[instrument(skip(self))]
    async fn guid(&self, guid: &str) -> Result<Option<Self::Item>, Error> {
        let pool = self.pool.clone();
        let guid_owned = guid.to_string();

        spawn_blocking(move || {
            tracing::debug!("fetching commodity by guid from sqlite");

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
        .map_err(|e| Error::Internal(format!("Join error in spawn_blocking: {e}")))?
    }

    #[instrument(skip(self))]
    async fn namespace(&self, namespace: &str) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();
        let namespace_owned = namespace.to_string();

        tokio::task::spawn_blocking(move || {
            tracing::debug!("fetching commodities by namespace from sqlite");

            let conn = pool.get()?;

            let sql = format!("{SEL}\nWHERE namespace = ?");
            let mut stmt = conn
                .prepare_cached(&sql)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;

            let rows = stmt.query_map([namespace_owned], |row| Self::Item::try_from(row))?;

            let items = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(count = items.len(), "commodities found by namespace");
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
    fn test_commodity_schemas() {
        let _ = sqlx::query_as!(
            Commodity,
            r"
				SELECT
				guid,
				namespace,
				mnemonic,
				fullname,
				cusip,
				fraction,
				quote_flag,
				quote_source,
				quote_tz
				FROM commodities
				",
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
    async fn test_commodity() {
        let query = setup().await;
        let result = query
            .guid("346629655191dcf59a7e2c2a85b70f69")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result.guid(), "346629655191dcf59a7e2c2a85b70f69");
        assert_eq!(result.namespace(), "CURRENCY");
        assert_eq!(result.mnemonic(), "EUR");
        assert_eq!(result.fullname(), "Euro");
        assert_eq!(result.cusip(), "978");
        assert_eq!(result.fraction(), 100);
        assert_eq!(result.quote_flag(), true);
        assert_eq!(result.quote_source(), "currency");
        assert_eq!(result.quote_tz(), "");
    }

    #[test(tokio::test)]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test(tokio::test)]
    async fn test_guid() {
        let query = setup().await;
        let result = query
            .guid("346629655191dcf59a7e2c2a85b70f69")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.fullname.as_ref().unwrap(), "Euro");
    }

    #[test(tokio::test)]
    async fn test_namespace() {
        let query = setup().await;
        let result = query.namespace("CURRENCY").await.unwrap();
        assert_eq!(result.len(), 4);
    }
}
