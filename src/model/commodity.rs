#[derive(Clone, Debug)]
#[cfg_attr(
    any(feature = "sqlite", feature = "postgres", feature = "mysql",),
    derive(sqlx::FromRow)
)]
pub struct Commodity {
    pub guid: String,
    pub namespace: String,
    pub mnemonic: String,
    pub fullname: Option<String>,
    pub cusip: Option<String>,
    pub fraction: i32,
    pub quote_flag: i32,
    pub quote_source: Option<String>,
    pub quote_tz: Option<String>,
}

impl<'q> Commodity {
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
            namespace,
            mnemonic,
            fullname,
            cusip,
            fraction as "fraction: i32",
            quote_flag as "quote_flag: i32",
            quote_source,
            quote_tz
            FROM commodities
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
            namespace,
            mnemonic,
            fullname,
            cusip,
            fraction,
            quote_flag,
            quote_source,
            quote_tz
            FROM commodities
            "#,
        )
    }

    #[cfg(any(feature = "sqlite", feature = "mysql",))]
    pub(crate) fn query_by_guid_question_mark<DB, O, T>(
        guid: T,
    ) -> sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::HasArguments<'q>>::Arguments>
    where
        DB: sqlx::Database,
        O: Send + Unpin + for<'r> sqlx::FromRow<'r, DB::Row>,
        T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        sqlx::query_as(
            r#"
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
            WHERE guid = ?
            "#,
        )
        .bind(guid)
    }

    #[cfg(any(feature = "postgres"))]
    pub(crate) fn query_by_guid_money_mark<DB, O, T>(
        guid: T,
    ) -> sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::HasArguments<'q>>::Arguments>
    where
        DB: sqlx::Database,
        O: Send + Unpin + for<'r> sqlx::FromRow<'r, DB::Row>,
        T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        sqlx::query_as(
            r#"
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
            WHERE guid = $1
            "#,
        )
        .bind(guid)
    }

    #[cfg(any(feature = "sqlite", feature = "mysql",))]
    pub(crate) fn query_by_namespace_question_mark<DB, O, T>(
        namespace: T,
    ) -> sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::HasArguments<'q>>::Arguments>
    where
        DB: sqlx::Database,
        O: Send + Unpin + for<'r> sqlx::FromRow<'r, DB::Row>,
        T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        sqlx::query_as(
            r#"
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
            WHERE namespace = ?
            "#,
        )
        .bind(namespace)
    }

    #[cfg(any(feature = "postgres"))]
    pub(crate) fn query_by_namespace_money_mark<DB, O, T>(
        namespace: T,
    ) -> sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::HasArguments<'q>>::Arguments>
    where
        DB: sqlx::Database,
        O: Send + Unpin + for<'r> sqlx::FromRow<'r, DB::Row>,
        T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    {
        sqlx::query_as(
            r#"
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
            WHERE namespace = $1
            "#,
        )
        .bind(namespace)
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

        const URI: &str = "sqlite://tests/db/sqlite/complex_sample.gnucash";
        type DB = sqlx::Sqlite;

        fn setup(uri: &str) -> sqlx::Pool<DB> {
            block_on(async {
                sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&format!("{}?mode=ro", uri)) // read only
                    .await
                    .unwrap()
            })
        }

        #[test]
        fn query() {
            let pool = setup(URI);
            let result: Vec<Commodity> =
                block_on(async { Commodity::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Commodity = block_on(async {
                Commodity::query_by_guid_question_mark("346629655191dcf59a7e2c2a85b70f69")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Euro", result.fullname.unwrap());
        }

        #[test]
        fn query_by_namespace() {
            let pool = setup(URI);
            let result: Vec<Commodity> = block_on(async {
                Commodity::query_by_namespace_question_mark("CURRENCY")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(4, result.len());
        }
    }

    #[cfg(feature = "postgres")]
    mod postgresql {
        use super::*;

        const URI: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
        type DB = sqlx::Postgres;

        fn setup(uri: &str) -> sqlx::Pool<DB> {
            block_on(async {
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .connect(uri)
                    .await
                    .unwrap()
            })
        }

        #[test]
        fn query() {
            let pool = setup(URI);
            let result: Vec<Commodity> =
                block_on(async { Commodity::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Commodity = block_on(async {
                Commodity::query_by_guid_money_mark("346629655191dcf59a7e2c2a85b70f69")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Euro", result.fullname.unwrap());
        }

        #[test]
        fn query_by_namespace() {
            let pool = setup(URI);
            let result: Vec<Commodity> = block_on(async {
                Commodity::query_by_namespace_money_mark("CURRENCY")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(4, result.len());
        }
    }

    #[cfg(feature = "mysql")]
    mod mysql {
        use super::*;

        const URI: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
        type DB = sqlx::MySql;

        fn setup(uri: &str) -> sqlx::Pool<DB> {
            block_on(async {
                sqlx::mysql::MySqlPoolOptions::new()
                    .max_connections(5)
                    .connect(uri)
                    .await
                    .unwrap()
            })
        }

        #[test]
        fn query() {
            let pool = setup(URI);
            let result: Vec<Commodity> =
                block_on(async { Commodity::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Commodity = block_on(async {
                Commodity::query_by_guid_question_mark("346629655191dcf59a7e2c2a85b70f69")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Euro", result.fullname.unwrap());
        }

        #[test]
        fn query_by_namespace() {
            let pool = setup(URI);
            let result: Vec<Commodity> = block_on(async {
                Commodity::query_by_namespace_question_mark("CURRENCY")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(4, result.len());
        }
    }
}
