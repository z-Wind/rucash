// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use chrono::{DateTime, NaiveDateTime};
use rusqlite::Row;
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;
use tokio::task::spawn_blocking;
use tracing::instrument;

use super::SQLiteQuery;
use crate::error::Error;
use crate::query::{SplitQ, SplitT};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Split {
    pub guid: String,
    pub tx_guid: String,
    pub account_guid: String,
    pub memo: String,
    pub action: String,
    pub reconcile_state: String,
    pub reconcile_date: Option<NaiveDateTime>,
    pub value_num: i64,
    pub value_denom: i64,
    pub quantity_num: i64,
    pub quantity_denom: i64,
    pub lot_guid: Option<String>,
}

impl<'a> TryFrom<&'a Row<'a>> for Split {
    type Error = rusqlite::Error;

    fn try_from(row: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            guid: row.get(0)?,
            tx_guid: row.get(1)?,
            account_guid: row.get(2)?,
            memo: row.get(3)?,
            action: row.get(4)?,
            reconcile_state: row.get(5)?,
            reconcile_date: row.get(6)?,
            value_num: row.get(7)?,
            value_denom: row.get(8)?,
            quantity_num: row.get(9)?,
            quantity_denom: row.get(10)?,
            lot_guid: row.get(11)?,
        })
    }
}

impl SplitT for Split {
    fn guid(&self) -> &str {
        &self.guid
    }
    fn tx_guid(&self) -> &str {
        &self.tx_guid
    }
    fn account_guid(&self) -> &str {
        &self.account_guid
    }
    fn memo(&self) -> &str {
        &self.memo
    }
    fn action(&self) -> &str {
        &self.action
    }
    fn reconcile_state(&self) -> bool {
        self.reconcile_state == "y" || self.reconcile_state == "Y"
    }
    fn reconcile_datetime(&self) -> Option<NaiveDateTime> {
        let datetime = self.reconcile_date?;
        if datetime.and_utc() == DateTime::UNIX_EPOCH {
            return None;
        }
        Some(datetime)
    }
    fn lot_guid(&self) -> &str {
        self.lot_guid.as_deref().unwrap_or_default()
    }

    #[cfg(not(feature = "decimal"))]
    #[allow(clippy::cast_precision_loss)]
    fn value(&self) -> f64 {
        self.value_num as f64 / self.value_denom as f64
    }

    #[cfg(feature = "decimal")]
    fn value(&self) -> Decimal {
        Decimal::new(self.value_num, 0) / Decimal::new(self.value_denom, 0)
    }

    #[cfg(not(feature = "decimal"))]
    #[allow(clippy::cast_precision_loss)]
    fn quantity(&self) -> f64 {
        self.quantity_num as f64 / self.quantity_denom as f64
    }

    #[cfg(feature = "decimal")]
    fn quantity(&self) -> Decimal {
        Decimal::new(self.quantity_num, 0) / Decimal::new(self.quantity_denom, 0)
    }
}

const SEL: &str = r"
SELECT
guid,
tx_guid,
account_guid,
memo,
action,
reconcile_state,
reconcile_date,
value_num,
value_denom,
quantity_num,
quantity_denom,
lot_guid
FROM splits
";

impl SplitQ for SQLiteQuery {
    type Item = Split;

    #[instrument(skip(self))]
    async fn all(&self) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();

        spawn_blocking(move || {
            tracing::debug!("fetching all splits from sqlite");

            let conn = pool.get()?;

            let mut stmt = conn
                .prepare_cached(SEL)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;

            let rows = stmt.query_map([], |row| Self::Item::try_from(row))?;

            let items = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(count = items.len(), "splits fetched from sqlite");
            Ok(items)
        })
        .await
        .map_err(|e| Error::Internal(format!("Join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn guid(&self, guid: &str) -> Result<Option<Self::Item>, Error> {
        let pool = self.pool.clone();
        let guid_owned = guid.to_string();

        tokio::task::spawn_blocking(move || {
            tracing::debug!("fetching split by guid from sqlite");

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
    async fn account(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();
        let guid_owned = guid.to_string();

        tokio::task::spawn_blocking(move || {
            tracing::debug!("fetching splits by account_guid from sqlite");

            let conn = pool.get()?;

            let sql = format!("{SEL}\nWHERE account_guid = ?");
            let mut stmt = conn
                .prepare_cached(&sql)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;

            let rows = stmt.query_map([guid_owned], |row| Self::Item::try_from(row))?;

            let items: Vec<Split> = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(count = items.len(), "splits found by account_guid");
            Ok(items)
        })
        .await
        .map_err(|e| Error::Internal(format!("Join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn transaction(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();
        let guid_owned = guid.to_string();

        tokio::task::spawn_blocking(move || {
            tracing::debug!("fetching splits by tx_guid from sqlite");

            let conn = pool.get()?;

            let sql = format!("{SEL}\nWHERE tx_guid = ?");
            let mut stmt = conn
                .prepare_cached(&sql)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;

            let rows = stmt.query_map([guid_owned], |row| Self::Item::try_from(row))?;

            let items = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(count = items.len(), "splits found by tx_guid");
            Ok(items)
        })
        .await
        .map_err(|e| Error::Internal(format!("Join error: {e}")))?
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    use pretty_assertions::assert_eq;
    use test_log::test;
    use tokio::sync::OnceCell;

    use super::*;

    #[cfg(feature = "schema")]
    // test schemas on compile time
    #[allow(dead_code)]
    fn test_split_schemas() {
        let _ = sqlx::query_as!(
            Split,
            r#"
				SELECT 	
				guid,
				tx_guid,
				account_guid,
				memo,
				action,
				reconcile_state,
				reconcile_date as "reconcile_date: NaiveDateTime",
				value_num,
				value_denom,
				quantity_num,
				quantity_denom,
				lot_guid
				FROM splits
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
    async fn test_split() {
        let query = setup().await;
        let result = query
            .guid("de832fe97e37811a7fff7e28b3a43425")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result.guid(), "de832fe97e37811a7fff7e28b3a43425");
        assert_eq!(result.tx_guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
        assert_eq!(result.account_guid(), "93fc043c3062aaa1297b30e543d2cd0d");
        assert_eq!(result.memo(), "");
        assert_eq!(result.action(), "");
        assert_eq!(result.reconcile_state(), false);
        assert_eq!(result.reconcile_datetime(), None);
        assert_eq!(result.lot_guid(), "");
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.value(), 150.0);
        #[cfg(feature = "decimal")]
        assert_eq!(result.value(), Decimal::new(150, 0));
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.quantity(), 150.0);
        #[cfg(feature = "decimal")]
        assert_eq!(result.quantity(), Decimal::new(150, 0));
    }

    #[test(tokio::test)]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 25);
    }

    #[test(tokio::test)]
    async fn test_guid() {
        let query = setup().await;
        let result = query
            .guid("de832fe97e37811a7fff7e28b3a43425")
            .await
            .unwrap()
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.value(), 150.0);
        #[cfg(feature = "decimal")]
        assert_eq!(result.value(), Decimal::new(150, 0));
    }

    #[test(tokio::test)]
    async fn test_account_guid() {
        let query = setup().await;
        let result = query
            .account("93fc043c3062aaa1297b30e543d2cd0d")
            .await
            .unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test(tokio::test)]
    async fn test_tx_guid() {
        let query = setup().await;
        let result = query
            .transaction("6c8876003c4a6026e38e3afb67d6f2b1")
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
    }
}
