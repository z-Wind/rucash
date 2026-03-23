// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use sqlx::AssertSqlSafe;
use tracing::instrument;

use crate::error::Error;
use crate::query::mysql::MySQLQuery;
use crate::query::{CommodityQ, CommodityT};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash, sqlx::FromRow)]
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

impl CommodityQ for MySQLQuery {
    type Item = Commodity;

    #[instrument(skip(self))]
    async fn all(&self) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching all commodities from mysql");
        sqlx::query_as(SEL)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }

    #[instrument(skip(self))]
    async fn guid(&self, guid: &str) -> Result<Option<Self::Item>, Error> {
        tracing::debug!("fetching commodity by guid from mysql");
        sqlx::query_as(AssertSqlSafe(format!("{SEL}\nWHERE guid = ?")))
            .bind(guid)
            .fetch_optional(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }

    #[instrument(skip(self))]
    async fn namespace(&self, namespace: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching commodities by namespace from mysql");
        sqlx::query_as(AssertSqlSafe(format!("{SEL}\nWHERE namespace = ?")))
            .bind(namespace)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use pretty_assertions::assert_eq;
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

    static Q: OnceCell<MySQLQuery> = OnceCell::const_new();
    async fn setup() -> &'static MySQLQuery {
        Q.get_or_init(|| async {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";

            tracing::debug!("work_dir: {:?}", std::env::current_dir());
            MySQLQuery::new(&format!("{uri}?mode=ro")).await.unwrap()
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
