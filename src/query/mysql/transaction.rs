// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use chrono::NaiveDateTime;

use crate::error::Error;
use crate::query::mysql::MySQLQuery;
use crate::query::{TransactionQ, TransactionT};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash, sqlx::FromRow)]
pub struct Transaction {
    pub guid: String,
    pub currency_guid: String,
    pub num: String,
    pub post_date: Option<NaiveDateTime>,
    pub enter_date: Option<NaiveDateTime>,
    pub description: Option<String>,
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
        self.post_date.expect("transaction post_date should exist")
    }
    fn enter_datetime(&self) -> NaiveDateTime {
        self.enter_date
            .expect("transaction enter_date should exist")
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

impl TransactionQ for MySQLQuery {
    type T = Transaction;

    async fn all(&self) -> Result<Vec<Self::T>, Error> {
        sqlx::query_as(SEL)
            .fetch_all(&self.pool)
            .await
            .map_err(std::convert::Into::into)
    }

    async fn guid(&self, guid: &str) -> Result<Vec<Self::T>, Error> {
        sqlx::query_as(&format!("{SEL}\nWHERE guid = ?"))
            .bind(guid)
            .fetch_all(&self.pool)
            .await
            .map_err(std::convert::Into::into)
    }

    async fn currency_guid(&self, guid: &str) -> Result<Vec<Self::T>, Error> {
        sqlx::query_as(&format!("{SEL}\nWHERE currency_guid = ?"))
            .bind(guid)
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
    			post_date as "post_date: NaiveDateTime",
    			enter_date as "enter_date: NaiveDateTime",
    			description
    			FROM transactions
    			"#,
        );
    }

    static Q: OnceCell<MySQLQuery> = OnceCell::const_new();
    async fn setup() -> &'static MySQLQuery {
        Q.get_or_init(|| async {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";

            println!("work_dir: {:?}", std::env::current_dir());
            MySQLQuery::new(&format!("{uri}?mode=ro")).await.unwrap()
        })
        .await
    }

    #[tokio::test]
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

    #[tokio::test]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 11);
    }

    #[tokio::test]
    async fn test_by_guid() {
        let query = setup().await;
        let result = query
            .guid("6c8876003c4a6026e38e3afb67d6f2b1")
            .await
            .unwrap();

        assert_eq!(
            result[0].post_date.unwrap(),
            NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );

        assert_eq!(
            result[0].enter_date.unwrap(),
            NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S").unwrap()
        );
    }

    #[tokio::test]
    async fn test_currency_guid() {
        let query = setup().await;
        let result = query
            .currency_guid("346629655191dcf59a7e2c2a85b70f69")
            .await
            .unwrap();

        assert_eq!(result.len(), 11);
    }
}
