#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Transaction {
    pub guid: String,
    pub currency_guid: String,
    pub num: String,
    pub post_date: Option<chrono::NaiveDateTime>,
    pub enter_date: Option<chrono::NaiveDateTime>,
    pub description: Option<String>,
}

impl<'q> Transaction {
    // test schemas on compile time
    #[allow(dead_code)]
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
            currency_guid,
            num,
            post_date,
            enter_date,
            description
            FROM transactions
            WHERE guid = ?
            "#,
        )
        .bind(guid)
    }

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
            currency_guid,
            num,
            post_date,
            enter_date,
            description
            FROM transactions
            WHERE guid = $1
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_currency_guid_question_mark<DB, O, T>(
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
            currency_guid,
            num,
            post_date,
            enter_date,
            description
            FROM transactions
            WHERE currency_guid = ?
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_currency_guid_money_mark<DB, O, T>(
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
            currency_guid,
            num,
            post_date,
            enter_date,
            description
            FROM transactions
            WHERE currency_guid = $1
            "#,
        )
        .bind(guid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    mod sqlite {
        use super::*;
        use chrono::NaiveDateTime;

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
            let result: Vec<Transaction> =
                block_on(async { Transaction::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(11, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Transaction = block_on(async {
                Transaction::query_by_guid_question_mark("6c8876003c4a6026e38e3afb67d6f2b1")
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
            let pool = setup(URI);
            let result: Vec<Transaction> = block_on(async {
                Transaction::query_by_currency_guid_question_mark(
                    "346629655191dcf59a7e2c2a85b70f69",
                )
                .fetch_all(&pool)
                .await
            })
            .unwrap();
            assert_eq!(11, result.len());
        }
    }
    mod postgresql {
        use super::*;
        use chrono::NaiveDateTime;

        const URI: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
        type DB = sqlx::Postgres;

        fn setup(uri: &str) -> sqlx::Pool<DB> {
            block_on(async {
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .connect(uri) // read only
                    .await
                    .unwrap()
            })
        }

        #[test]
        fn query() {
            let pool = setup(URI);
            let result: Vec<Transaction> =
                block_on(async { Transaction::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(11, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Transaction = block_on(async {
                Transaction::query_by_guid_money_mark("6c8876003c4a6026e38e3afb67d6f2b1")
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
            let pool = setup(URI);
            let result: Vec<Transaction> = block_on(async {
                Transaction::query_by_currency_guid_money_mark("346629655191dcf59a7e2c2a85b70f69")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(11, result.len());
        }
    }

    mod mysql {
        use super::*;
        use chrono::NaiveDateTime;

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
            let result: Vec<Transaction> =
                block_on(async { Transaction::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(11, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Transaction = block_on(async {
                Transaction::query_by_guid_question_mark("6c8876003c4a6026e38e3afb67d6f2b1")
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
            let pool = setup(URI);
            let result: Vec<Transaction> = block_on(async {
                Transaction::query_by_currency_guid_question_mark(
                    "346629655191dcf59a7e2c2a85b70f69",
                )
                .fetch_all(&pool)
                .await
            })
            .unwrap();
            assert_eq!(11, result.len());
        }
    }
}
