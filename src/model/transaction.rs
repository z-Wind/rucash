#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::kind::SQLKind;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
    fn test_schemas() -> sqlx::query::Map<
        'q,
        sqlx::Sqlite,
        fn(sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error>,
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

#[cfg(test)]
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
mod tests {
    use super::*;
    use futures::executor::block_on;
    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;
        use chrono::NaiveDateTime;

        const URI: &str = "sqlite://tests/db/sqlite/complex_sample.gnucash";
        type DB = sqlx::Sqlite;

        fn setup(uri: &str) -> (sqlx::Pool<DB>, SQLKind) {
            (
                block_on(async {
                    sqlx::sqlite::SqlitePoolOptions::new()
                        .max_connections(5)
                        .connect(&format!("{}?mode=ro", uri)) // read only
                        .await
                        .unwrap()
                }),
                uri.parse().expect("sqlite"),
            )
        }

        #[test]
        fn query() {
            let (pool, _kind) = setup(URI);
            let result: Vec<Transaction> =
                block_on(async { Transaction::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(11, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup(URI);
            let result: Transaction = block_on(async {
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
            let (pool, kind) = setup(URI);
            let result: Vec<Transaction> = block_on(async {
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

        const URI: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
        type DB = sqlx::Postgres;

        fn setup(uri: &str) -> (sqlx::Pool<DB>, SQLKind) {
            (
                block_on(async {
                    sqlx::postgres::PgPoolOptions::new()
                        .max_connections(5)
                        .connect(uri)
                        .await
                        .unwrap()
                }),
                uri.parse().expect("postgres"),
            )
        }

        #[test]
        fn query() {
            let (pool, _kind) = setup(URI);
            let result: Vec<Transaction> =
                block_on(async { Transaction::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(11, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup(URI);
            let result: Transaction = block_on(async {
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
            let (pool, kind) = setup(URI);
            let result: Vec<Transaction> = block_on(async {
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

        const URI: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
        type DB = sqlx::MySql;

        fn setup(uri: &str) -> (sqlx::Pool<DB>, SQLKind) {
            (
                block_on(async {
                    sqlx::mysql::MySqlPoolOptions::new()
                        .max_connections(5)
                        .connect(uri)
                        .await
                        .unwrap()
                }),
                uri.parse().expect("mysql"),
            )
        }

        #[test]
        fn query() {
            let (pool, _kind) = setup(URI);
            let result: Vec<Transaction> =
                block_on(async { Transaction::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(11, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup(URI);
            let result: Transaction = block_on(async {
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
            let (pool, kind) = setup(URI);
            let result: Vec<Transaction> = block_on(async {
                Transaction::query_by_currency_guid("346629655191dcf59a7e2c2a85b70f69", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(11, result.len());
        }
    }
}
