#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
use super::TestSchemas;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::kind::SQLKind;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
#[cfg_attr(
    any(feature = "sqlite", feature = "postgres", feature = "mysql",),
    derive(sqlx::FromRow)
)]
pub struct Transaction {
    pub guid: String,
    pub currency_guid: String,
    pub num: String,
    pub post_date: Option<chrono::NaiveDateTime>,
    pub enter_date: Option<chrono::NaiveDateTime>,
    pub description: Option<String>,
}

impl super::NullNone for Transaction {
    fn null_none(self) -> Self {
        let description = self.description.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });

        Self {
            description,
            ..self
        }
    }
}

impl<'q> Transaction {
    // test schemas on compile time
    #[allow(dead_code)]
    #[cfg(feature = "sqlite")]
    fn test_schemas() -> TestSchemas<
        'q,
        sqlx::Sqlite,
        sqlx::sqlite::SqliteRow,
        Self,
        sqlx::sqlite::SqliteArguments<'q>,
    > {
        sqlx::query_as!(
            Self,
            r#"
            SELECT
            guid,
            currency_guid,
            num,
            post_date as "post_date: chrono::NaiveDateTime",
            enter_date as "enter_date: chrono::NaiveDateTime",
            description
            FROM transactions
            "#,
        )
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
    pub(crate) fn query<DB, O>(
    ) -> sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::HasArguments<'q>>::Arguments>
    where
        DB: sqlx::Database,
        O: Send + Unpin + for<'r> sqlx::FromRow<'r, DB::Row>,
    {
        sqlx::query_as(
            r#"
            SELECT
            guid,
            currency_guid,
            num,
            post_date,
            enter_date,
            description
            FROM transactions
            "#,
        )
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
    pub(crate) fn query_by_guid<DB, O, T>(
        guid: T,
        kind: SQLKind,
    ) -> sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::HasArguments<'q>>::Arguments>
    where
        DB: sqlx::Database,
        O: Send + Unpin + for<'r> sqlx::FromRow<'r, DB::Row>,
        T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        match kind {
            SQLKind::Postgres => sqlx::query_as(
                r#"
                SELECT
                guid,
                currency_guid,
                num,
                post_date,
                enter_date,
                description
                FROM transactions
                WHERE guid = $1
                "#,
            )
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
                r#"
            SELECT
            guid,
            currency_guid,
            num,
            post_date,
            enter_date,
            description
            FROM transactions
            WHERE guid = ?
            "#,
            )
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
    pub(crate) fn query_by_currency_guid<DB, O, T>(
        guid: T,
        kind: SQLKind,
    ) -> sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::HasArguments<'q>>::Arguments>
    where
        DB: sqlx::Database,
        O: Send + Unpin + for<'r> sqlx::FromRow<'r, DB::Row>,
        T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        match kind {
            SQLKind::Postgres => sqlx::query_as(
                r#"
                SELECT
                guid,
                currency_guid,
                num,
                post_date,
                enter_date,
                description
                FROM transactions
                WHERE currency_guid = $1
                "#,
            )
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
                r#"
            SELECT
            guid,
            currency_guid,
            num,
            post_date,
            enter_date,
            description
            FROM transactions
            WHERE currency_guid = ?
            "#,
            )
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }
}

#[cfg(feature = "xml")]
use xmltree::Element;
#[cfg(feature = "xml")]
impl Transaction {
    pub(crate) fn new_by_element(e: &Element) -> Self {
        let guid = e
            .get_child("id")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("id must exist");
        let currency_guid = e
            .get_child("currency")
            .and_then(|x| x.get_child("id"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("currency must exist");
        let num = e
            .get_child("num")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .unwrap_or_default();
        let post_date = e
            .get_child("date-posted")
            .and_then(|x| x.get_child("date"))
            .and_then(|x| x.get_text())
            // .map(|x| x.into_owned())
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z")
                    .expect("date-posted must be %Y-%m-%d %H:%M:%S%z")
            });
        let enter_date = e
            .get_child("date-entered")
            .and_then(|x| x.get_child("date"))
            .and_then(|x| x.get_text())
            // .map(|x| x.into_owned())
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z")
                    .expect("date-entered must be %Y-%m-%d %H:%M:%S%z")
            });
        let description = e
            .get_child("description")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());

        Self {
            guid,
            currency_guid,
            num,
            post_date,
            enter_date,
            description,
        }
    }
}
#[cfg(test)]
#[cfg(any(
    feature = "sqlite",
    feature = "postgres",
    feature = "mysql",
    feature = "xml"
))]
mod tests {
    use super::*;
    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
    use tokio::runtime::Runtime;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;
        use chrono::NaiveDateTime;

        type DB = sqlx::Sqlite;

        fn setup() -> (sqlx::Pool<DB>, Runtime, SQLKind) {
            let uri: &str = &format!(
                "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            (
                rt.block_on(async {
                    sqlx::sqlite::SqlitePoolOptions::new()
                        .max_connections(5)
                        .connect(&format!("{}?mode=ro", uri)) // read only
                        .await
                        .unwrap()
                }),
                rt,
                uri.parse().expect("sqlite"),
            )
        }

        #[test]
        fn query() {
            let (pool, rt, _) = setup();
            let result: Vec<Transaction> = rt
                .block_on(async { Transaction::query().fetch_all(&pool).await })
                .unwrap();
            assert_eq!(11, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, rt, kind) = setup();
            let result: Transaction = rt
                .block_on(async {
                    Transaction::query_by_guid("6c8876003c4a6026e38e3afb67d6f2b1", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            assert_eq!(
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap(),
                result.post_date.unwrap()
            );

            assert_eq!(
                NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S").unwrap(),
                result.enter_date.unwrap()
            );
        }

        #[test]
        fn query_by_currency_guid() {
            let (pool, rt, kind) = setup();
            let result: Vec<Transaction> = rt
                .block_on(async {
                    Transaction::query_by_currency_guid("346629655191dcf59a7e2c2a85b70f69", kind)
                        .fetch_all(&pool)
                        .await
                })
                .unwrap();
            assert_eq!(11, result.len());
        }
    }

    #[cfg(feature = "postgres")]
    mod postgresql {
        use super::*;
        use chrono::NaiveDateTime;

        type DB = sqlx::Postgres;

        fn setup() -> (sqlx::Pool<DB>, Runtime, SQLKind) {
            let uri: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            (
                rt.block_on(async {
                    sqlx::postgres::PgPoolOptions::new()
                        .max_connections(5)
                        .connect(uri)
                        .await
                        .unwrap()
                }),
                rt,
                uri.parse().expect("postgres"),
            )
        }

        #[test]
        fn query() {
            let (pool, rt, _) = setup();
            let result: Vec<Transaction> = rt
                .block_on(async { Transaction::query().fetch_all(&pool).await })
                .unwrap();
            assert_eq!(11, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, rt, kind) = setup();
            let result: Transaction = rt
                .block_on(async {
                    Transaction::query_by_guid("6c8876003c4a6026e38e3afb67d6f2b1", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            assert_eq!(
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap(),
                result.post_date.unwrap()
            );

            assert_eq!(
                NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S").unwrap(),
                result.enter_date.unwrap()
            );
        }

        #[test]
        fn query_by_currency_guid() {
            let (pool, rt, kind) = setup();
            let result: Vec<Transaction> = rt
                .block_on(async {
                    Transaction::query_by_currency_guid("346629655191dcf59a7e2c2a85b70f69", kind)
                        .fetch_all(&pool)
                        .await
                })
                .unwrap();
            assert_eq!(11, result.len());
        }
    }

    #[cfg(feature = "mysql")]
    mod mysql {
        use super::*;
        use chrono::NaiveDateTime;

        type DB = sqlx::MySql;

        fn setup() -> (sqlx::Pool<DB>, Runtime, SQLKind) {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            (
                rt.block_on(async {
                    sqlx::mysql::MySqlPoolOptions::new()
                        .max_connections(5)
                        .connect(uri)
                        .await
                        .unwrap()
                }),
                rt,
                uri.parse().expect("mysql"),
            )
        }

        #[test]
        fn query() {
            let (pool, rt, _) = setup();
            let result: Vec<Transaction> = rt
                .block_on(async { Transaction::query().fetch_all(&pool).await })
                .unwrap();
            assert_eq!(11, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, rt, kind) = setup();
            let result: Transaction = rt
                .block_on(async {
                    Transaction::query_by_guid("6c8876003c4a6026e38e3afb67d6f2b1", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            assert_eq!(
                NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap(),
                result.post_date.unwrap()
            );

            assert_eq!(
                NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S").unwrap(),
                result.enter_date.unwrap()
            );
        }

        #[test]
        fn query_by_currency_guid() {
            let (pool, rt, kind) = setup();
            let result: Vec<Transaction> = rt
                .block_on(async {
                    Transaction::query_by_currency_guid("346629655191dcf59a7e2c2a85b70f69", kind)
                        .fetch_all(&pool)
                        .await
                })
                .unwrap();
            assert_eq!(11, result.len());
        }
    }

    #[cfg(feature = "xml")]
    mod xml {
        use super::*;
        use std::sync::Arc;

        #[allow(dead_code)]
        fn setup() -> Arc<Element> {
            let uri: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );
            crate::XMLBook::new(uri).unwrap().pool.0
        }

        #[test]
        fn new_by_element() {
            let data = r##"
            <?xml version="1.0" encoding="utf-8" ?>
            <gnc-v2
                xmlns:gnc="http://www.gnucash.org/XML/gnc"
                xmlns:act="http://www.gnucash.org/XML/act"
                xmlns:book="http://www.gnucash.org/XML/book"
                xmlns:cd="http://www.gnucash.org/XML/cd"
                xmlns:cmdty="http://www.gnucash.org/XML/cmdty"
                xmlns:price="http://www.gnucash.org/XML/price"
                xmlns:slot="http://www.gnucash.org/XML/slot"
                xmlns:split="http://www.gnucash.org/XML/split"
                xmlns:sx="http://www.gnucash.org/XML/sx"
                xmlns:trn="http://www.gnucash.org/XML/trn"
                xmlns:ts="http://www.gnucash.org/XML/ts"
                xmlns:fs="http://www.gnucash.org/XML/fs"
                xmlns:bgt="http://www.gnucash.org/XML/bgt"
                xmlns:recurrence="http://www.gnucash.org/XML/recurrence"
                xmlns:lot="http://www.gnucash.org/XML/lot"
                xmlns:addr="http://www.gnucash.org/XML/addr"
                xmlns:billterm="http://www.gnucash.org/XML/billterm"
                xmlns:bt-days="http://www.gnucash.org/XML/bt-days"
                xmlns:bt-prox="http://www.gnucash.org/XML/bt-prox"
                xmlns:cust="http://www.gnucash.org/XML/cust"
                xmlns:employee="http://www.gnucash.org/XML/employee"
                xmlns:entry="http://www.gnucash.org/XML/entry"
                xmlns:invoice="http://www.gnucash.org/XML/invoice"
                xmlns:job="http://www.gnucash.org/XML/job"
                xmlns:order="http://www.gnucash.org/XML/order"
                xmlns:owner="http://www.gnucash.org/XML/owner"
                xmlns:taxtable="http://www.gnucash.org/XML/taxtable"
                xmlns:tte="http://www.gnucash.org/XML/tte"
                xmlns:vendor="http://www.gnucash.org/XML/vendor">
            <gnc:transaction version="2.0.0">
                <trn:id type="guid">6c8876003c4a6026e38e3afb67d6f2b1</trn:id>
                <trn:currency>
                    <cmdty:space>CURRENCY</cmdty:space>
                    <cmdty:id>EUR</cmdty:id>
                </trn:currency>
                <trn:date-posted>
                    <ts:date>2014-12-24 10:59:00 +0000</ts:date>
                </trn:date-posted>
                <trn:date-entered>
                    <ts:date>2014-12-25 10:08:15 +0000</ts:date>
                </trn:date-entered>
                <trn:description>income 1</trn:description>
                <trn:splits>
                    <trn:split>
                        <split:id type="guid">de832fe97e37811a7fff7e28b3a43425</split:id>
                        <split:reconciled-state>n</split:reconciled-state>
                        <split:value>15000/100</split:value>
                        <split:quantity>15000/100</split:quantity>
                        <split:account type="guid">93fc043c3062aaa1297b30e543d2cd0d</split:account>
                    </trn:split>
                    <trn:split>
                        <split:id type="guid">1e612f650eb598d9e803902b6aca73e3</split:id>
                        <split:reconciled-state>n</split:reconciled-state>
                        <split:value>-15000/100</split:value>
                        <split:quantity>-15000/100</split:quantity>
                        <split:account type="guid">6bbc8f20544452cac1637fb9a9b851bb</split:account>
                    </trn:split>
                </trn:splits>
            </gnc:transaction>
            </gnc-v2>
            "##;

            let e = Element::parse(data.as_bytes())
                .unwrap()
                .take_child("transaction")
                .unwrap();

            let transaction = Transaction::new_by_element(&e);

            assert_eq!(transaction.guid, "6c8876003c4a6026e38e3afb67d6f2b1");
            assert_eq!(transaction.currency_guid, "EUR");
            assert_eq!(transaction.num, "");
            assert_eq!(
                transaction
                    .post_date
                    .as_ref()
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                "2014-12-24 10:59:00"
            );
            assert_eq!(
                transaction
                    .enter_date
                    .as_ref()
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                "2014-12-25 10:08:15"
            );
            assert_eq!(transaction.description.as_ref().unwrap(), "income 1");
        }
    }
}
