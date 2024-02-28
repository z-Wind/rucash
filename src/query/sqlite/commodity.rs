// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use crate::error::Error;
use crate::query::sqlite::SQLiteQuery;
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
    fn guid(&self) -> String {
        self.guid.clone()
    }
    fn namespace(&self) -> String {
        self.namespace.clone()
    }
    fn mnemonic(&self) -> String {
        self.mnemonic.clone()
    }
    fn fullname(&self) -> String {
        self.fullname.clone().unwrap_or_default()
    }
    fn cusip(&self) -> String {
        self.cusip.clone().unwrap_or_default()
    }
    fn fraction(&self) -> i64 {
        self.fraction
    }
    fn quote_flag(&self) -> bool {
        self.quote_flag != 0
    }
    fn quote_source(&self) -> String {
        self.quote_source.clone().unwrap_or_default()
    }
    fn quote_tz(&self) -> String {
        self.quote_tz.clone().unwrap_or_default()
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
    type C = Commodity;

    async fn all(&self) -> Result<Vec<Self::C>, Error> {
        sqlx::query_as(SEL)
            .fetch_all(&self.pool)
            .await
            .map_err(std::convert::Into::into)
    }

    async fn guid(&self, guid: &str) -> Result<Vec<Self::C>, Error> {
        sqlx::query_as(&format!("{SEL}\nWHERE guid = ?"))
            .bind(guid)
            .fetch_all(&self.pool)
            .await
            .map_err(std::convert::Into::into)
    }

    async fn namespace(&self, namespace: &str) -> Result<Vec<Self::C>, Error> {
        sqlx::query_as(&format!("{SEL}\nWHERE namespace = ?"))
            .bind(namespace)
            .fetch_all(&self.pool)
            .await
            .map_err(std::convert::Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use tokio::sync::OnceCell;

    #[cfg(feature = "schema")]
    impl<'q> SQLiteQuery {
        // test schemas on compile time
        #[allow(dead_code)]
        async fn test_commodity_schemas(&self) {
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
            )
            .fetch_all(&self.pool)
            .await;
        }
    }

    static Q: OnceCell<SQLiteQuery> = OnceCell::const_new();
    async fn setup() -> &'static SQLiteQuery {
        Q.get_or_init(|| async {
            let uri: &str = &format!(
                "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            println!("work_dir: {:?}", std::env::current_dir());
            SQLiteQuery::new(&format!("{uri}?mode=ro")).await.unwrap()
        })
        .await
    }

    #[tokio::test]
    async fn test_commodity() {
        let query = setup().await;
        let result = query
            .guid("346629655191dcf59a7e2c2a85b70f69")
            .await
            .unwrap();

        let result = &result[0];
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

    #[tokio::test]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_guid() {
        let query = setup().await;
        let result = query
            .guid("346629655191dcf59a7e2c2a85b70f69")
            .await
            .unwrap();
        assert_eq!(result[0].fullname.as_ref().unwrap(), "Euro");
    }

    #[tokio::test]
    async fn test_namespace() {
        let query = setup().await;
        let result = query.namespace("CURRENCY").await.unwrap();
        assert_eq!(result.len(), 4);
    }
}
