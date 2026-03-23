// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use chrono::NaiveDateTime;
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;
use sqlx::AssertSqlSafe;
use tracing::instrument;

use crate::error::Error;
use crate::query::postgresql::PostgreSQLQuery;
use crate::query::{PriceQ, PriceT};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash, sqlx::FromRow)]
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

impl PriceQ for PostgreSQLQuery {
    type Item = Price;

    #[instrument(skip(self))]
    async fn all(&self) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching all prices from postgresql");
        sqlx::query_as(SEL)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }
    #[instrument(skip(self))]
    async fn guid(&self, guid: &str) -> Result<Option<Self::Item>, Error> {
        tracing::debug!("fetching price by guid from postgresql");
        sqlx::query_as(AssertSqlSafe(format!("{SEL}\nWHERE guid = $1")))
            .bind(guid)
            .fetch_optional(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }
    #[instrument(skip(self))]
    async fn commodity(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching prices by commodity_guid from postgresql");
        sqlx::query_as(AssertSqlSafe(format!("{SEL}\nWHERE commodity_guid = $1")))
            .bind(guid)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }
    #[instrument(skip(self))]
    async fn currency(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching prices by currency_guid from postgresql");
        sqlx::query_as(AssertSqlSafe(format!("{SEL}\nWHERE currency_guid = $1")))
            .bind(guid)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }
    #[instrument(skip(self))]
    async fn commodity_or_currency(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching prices by commodity or currency guid from postgresql");
        sqlx::query_as(AssertSqlSafe(format!(
            "{SEL}\nWHERE commodity_guid = $1 OR currency_guid = $1"
        )))
        .bind(guid)
        .bind(guid)
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
        .map_err(std::convert::Into::into)
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

    static Q: OnceCell<PostgreSQLQuery> = OnceCell::const_new();
    async fn setup() -> &'static PostgreSQLQuery {
        Q.get_or_init(|| async {
            let uri: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";

            tracing::debug!("work_dir: {:?}", std::env::current_dir());
            PostgreSQLQuery::new(&format!("{uri}?mode=ro"))
                .await
                .unwrap()
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
