// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use sqlx::AssertSqlSafe;
use tracing::instrument;

use crate::error::Error;
use crate::query::postgresql::PostgreSQLQuery;
use crate::query::{AccountQ, AccountT};

#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash, sqlx::FromRow)]
pub struct Account {
    pub(crate) guid: String,
    pub(crate) name: String,
    pub(crate) account_type: String,
    pub(crate) commodity_guid: Option<String>,
    pub(crate) commodity_scu: i32,
    pub(crate) non_std_scu: i32,
    pub(crate) parent_guid: Option<String>,
    pub(crate) code: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) hidden: Option<i32>,
    pub(crate) placeholder: Option<i32>,
}

impl AccountT for Account {
    fn guid(&self) -> &str {
        &self.guid
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn account_type(&self) -> &str {
        &self.account_type
    }
    fn commodity_guid(&self) -> &str {
        self.commodity_guid.as_deref().unwrap_or_default()
    }
    fn commodity_scu(&self) -> i64 {
        i64::from(self.commodity_scu)
    }
    fn non_std_scu(&self) -> bool {
        self.non_std_scu != 0
    }
    fn parent_guid(&self) -> &str {
        self.parent_guid.as_deref().unwrap_or_default()
    }
    fn code(&self) -> &str {
        self.code.as_deref().unwrap_or_default()
    }
    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or_default()
    }
    fn hidden(&self) -> bool {
        self.hidden.is_some_and(|x| x != 0)
    }
    fn placeholder(&self) -> bool {
        self.placeholder.is_some_and(|x| x != 0)
    }
}

const SEL: &str = r"
SELECT 
guid, 
name,
account_type,
commodity_guid,
commodity_scu,
non_std_scu,
parent_guid,
code,
description,
hidden,
placeholder
FROM accounts
";

impl AccountQ for PostgreSQLQuery {
    type Item = Account;

    #[instrument(skip(self))]
    async fn all(&self) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching all accounts from postgresql");
        sqlx::query_as(SEL)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }

    #[instrument(skip(self))]
    async fn guid(&self, guid: &str) -> Result<Option<Self::Item>, Error> {
        tracing::debug!("fetching account by guid from postgresql");
        sqlx::query_as(AssertSqlSafe(format!("{SEL}\nWHERE guid = $1")))
            .bind(guid)
            .fetch_optional(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }

    #[instrument(skip(self))]
    async fn commodity(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching accounts by commodity_guid from postgresql");
        sqlx::query_as(AssertSqlSafe(format!("{SEL}\nWHERE commodity_guid = $1")))
            .bind(guid)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }

    #[instrument(skip(self))]
    async fn parent(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching accounts by parent_guid from postgresql");
        sqlx::query_as(AssertSqlSafe(format!("{SEL}\nWHERE parent_guid = $1")))
            .bind(guid)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }

    async fn name(&self, name: &str) -> Result<Vec<Self::Item>, Error> {
        sqlx::query_as(AssertSqlSafe(format!("{SEL}\nWHERE name = $1")))
            .bind(name)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
            .map_err(std::convert::Into::into)
    }

    #[instrument(skip(self))]
    async fn contains_name_ignore_case(&self, name: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("searching accounts with name pattern from postgresql");
        let name = format!("%{name}%");
        sqlx::query_as(AssertSqlSafe(format!(
            "{SEL}\nWHERE LOWER(name) LIKE LOWER($1)"
        )))
        .bind(name)
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| tracing::error!("failed to execute query: {e}"))
        .map_err(std::convert::Into::into)
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
    fn test_account_schemas() {
        let _ = sqlx::query_as!(
            Account,
            r"
    			SELECT
    			guid,
    			name,
    			account_type,
    			commodity_guid,
    			commodity_scu,
    			non_std_scu,
    			parent_guid,
    			code,
    			description,
    			hidden,
    			placeholder
    			FROM accounts
    			"
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
    async fn test_account() {
        let query = setup().await;
        let result = query
            .guid("fcd795021c976ba75621ec39e75f6214")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result.guid(), "fcd795021c976ba75621ec39e75f6214");
        assert_eq!(result.name(), "Asset");
        assert_eq!(result.account_type(), "ASSET");
        assert_eq!(result.commodity_guid(), "346629655191dcf59a7e2c2a85b70f69");
        assert_eq!(result.commodity_scu(), 100);
        assert_eq!(result.non_std_scu(), false);
        assert_eq!(result.parent_guid(), "00622dda21937b29e494179de5013f82");
        assert_eq!(result.code(), "");
        assert_eq!(result.description(), "");
        assert_eq!(result.hidden(), false);
        assert_eq!(result.placeholder(), true);
    }

    #[test(tokio::test)]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 21);
    }

    #[test(tokio::test)]
    async fn test_guid() {
        let query = setup().await;
        let result = query
            .guid("fcd795021c976ba75621ec39e75f6214")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result.name, "Asset");
    }

    #[test(tokio::test)]
    async fn test_commodity_guid() {
        let query = setup().await;
        let result = query
            .commodity("346629655191dcf59a7e2c2a85b70f69")
            .await
            .unwrap();
        assert_eq!(result.len(), 14);
    }

    #[test(tokio::test)]
    async fn test_parent_guid() {
        let query = setup().await;
        let result = query
            .parent("fcd795021c976ba75621ec39e75f6214")
            .await
            .unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test(tokio::test)]
    async fn test_name() {
        let query = setup().await;
        let result = query.name("Asset").await.unwrap();
        assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
    }

    #[test(tokio::test)]
    async fn test_contains_name_ignore_case() {
        let query = setup().await;
        let result = query.contains_name_ignore_case("AS").await.unwrap();
        assert_eq!(result.len(), 3);
    }
}
