// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use chrono::NaiveDateTime;
use rusqlite::Row;
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;
use tokio::task::spawn_blocking;
use tracing::instrument;

use super::SQLiteQuery;
use crate::error::Error;
use crate::query::{PriceQ, PriceT};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Price {
    pub guid: String,
    pub commodity_guid: String,
    pub currency_guid: String,
    pub date: NaiveDateTime,
    pub source: Option<String>,
    pub r#type: Option<String>,
    pub value_num: i64,
    pub value_denom: i64,
}

impl<'a> TryFrom<&'a Row<'a>> for Price {
    type Error = rusqlite::Error;

    fn try_from(row: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            guid: row.get(0)?,
            commodity_guid: row.get(1)?,
            currency_guid: row.get(2)?,
            date: row.get(3)?,
            source: row.get(4)?,
            r#type: row.get(5)?,
            value_num: row.get(6)?,
            value_denom: row.get(7)?,
        })
    }
}

impl PriceT for Price {
    fn guid(&self) -> &str {
        &self.guid
    }
    fn commodity_guid(&self) -> &str {
        &self.commodity_guid
    }
    fn currency_guid(&self) -> &str {
        &self.currency_guid
    }
    fn datetime(&self) -> NaiveDateTime {
        self.date
    }
    fn source(&self) -> &str {
        self.source.as_deref().unwrap_or_default()
    }
    fn r#type(&self) -> &str {
        self.r#type.as_deref().unwrap_or_default()
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
}

const SEL: &str = r"
SELECT
guid,
commodity_guid,
currency_guid,
date,
source,
type,
value_num,
value_denom
FROM prices
";

impl PriceQ for SQLiteQuery {
    type Item = Price;

    #[instrument(skip(self))]
    async fn all(&self) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();

        spawn_blocking(move || {
            tracing::debug!("fetching all prices from sqlite");

            let conn = pool.get()?;

            let mut stmt = conn
                .prepare_cached(SEL)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;

            let rows = stmt.query_map([], |row| Self::Item::try_from(row))?;

            let items = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(count = items.len(), "prices fetched from sqlite");
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
            tracing::debug!("fetching price by guid from sqlite");

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
    async fn commodity(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();
        let guid_owned = guid.to_string();

        tokio::task::spawn_blocking(move || {
            tracing::debug!("fetching prices by commodity_guid from sqlite");

            let conn = pool.get()?;

            let sql = format!("{SEL}\nWHERE commodity_guid = ?");
            let mut stmt = conn
                .prepare_cached(&sql)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;
            let rows = stmt.query_map([guid_owned], |row| Self::Item::try_from(row))?;

            let items = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(count = items.len(), "prices found by commodity_guid");
            Ok(items)
        })
        .await
        .map_err(|e| Error::Internal(format!("Join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn currency(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();
        let guid_owned = guid.to_string();

        tokio::task::spawn_blocking(move || {
            tracing::debug!("fetching prices by currency_guid from sqlite");

            let conn = pool.get()?;

            let sql = format!("{SEL}\nWHERE currency_guid = ?");
            let mut stmt = conn
                .prepare_cached(&sql)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;
            let rows = stmt.query_map([guid_owned], |row| Self::Item::try_from(row))?;

            let items = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(count = items.len(), "prices found by currency_guid");
            Ok(items)
        })
        .await
        .map_err(|e| Error::Internal(format!("Join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn commodity_or_currency(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        let pool = self.pool.clone();
        let guid_owned = guid.to_string();

        tokio::task::spawn_blocking(move || {
            tracing::debug!("fetching prices by commodity or currency guid from sqlite");

            let conn = pool.get()?;

            let sql = format!("{SEL}\nWHERE commodity_guid = ? OR currency_guid = ?");
            let mut stmt = conn
                .prepare_cached(&sql)
                .inspect_err(|e| tracing::error!("failed to prepare statement: {e}"))?;

            let rows =
                stmt.query_map([&guid_owned, &guid_owned], |row| Self::Item::try_from(row))?;

            let items = rows
                .collect::<Result<Vec<_>, _>>()
                .inspect_err(|e| tracing::error!("failed to collect rows: {e}"))?;

            tracing::debug!(
                count = items.len(),
                "prices found by commodity or currency guid"
            );
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
    fn test_price_schemas() {
        let _ = sqlx::query_as!(
            Price,
            r#"
				SELECT
				guid,
				commodity_guid,
				currency_guid,
				date as "date: NaiveDateTime",
				source,
				type,
				value_num,
				value_denom
				FROM prices
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
    async fn test_price() {
        let query = setup().await;
        let result = query
            .guid("0d6684f44fb018e882de76094ed9c433")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result.guid(), "0d6684f44fb018e882de76094ed9c433");
        assert_eq!(result.commodity_guid(), "d821d6776fde9f7c2d01b67876406fd3");
        assert_eq!(result.currency_guid(), "5f586908098232e67edb1371408bfaa8");
        assert_eq!(
            result.datetime(),
            NaiveDateTime::parse_from_str("2018-02-20 23:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(result.source(), "user:price-editor");
        assert_eq!(result.r#type(), "unknown");
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result.value(), Decimal::new(15, 1));
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
            .guid("0d6684f44fb018e882de76094ed9c433")
            .await
            .unwrap()
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result.value(), Decimal::new(15, 1));
    }

    #[test(tokio::test)]
    async fn test_commodity_guid() {
        let query = setup().await;
        let result = query
            .commodity("d821d6776fde9f7c2d01b67876406fd3")
            .await
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result[0].value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result[0].value(), Decimal::new(15, 1));
    }

    #[test(tokio::test)]
    async fn test_currency_guid() {
        let query = setup().await;
        let result = query
            .currency("5f586908098232e67edb1371408bfaa8")
            .await
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result[0].value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result[0].value(), Decimal::new(15, 1));
    }

    #[test(tokio::test)]
    async fn test_commodity_or_currency_guid() {
        let query = setup().await;
        let result = query
            .commodity_or_currency("5f586908098232e67edb1371408bfaa8")
            .await
            .unwrap();
        assert_eq!(result.len(), 4);
    }
}
