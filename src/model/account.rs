#[derive(Clone, Debug)]
#[cfg_attr(
    any(feature = "sqlite", feature = "postgres", feature = "mysql",),
    derive(sqlx::FromRow)
)]
pub struct Account {
    pub guid: String,
    pub name: String,
    pub account_type: String,
    pub commodity_guid: Option<String>,
    pub commodity_scu: i32,
    pub non_std_scu: i32,
    pub parent_guid: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub hidden: Option<i32>,
    pub placeholder: Option<i32>,
}

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
impl<'q> Account {
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
            name,
            account_type,
            commodity_guid,
            commodity_scu as "commodity_scu: i32",
            non_std_scu as "non_std_scu: i32",
            parent_guid,
            code,
            description,
            hidden as "hidden: i32",
            placeholder as "placeholder: i32"
            FROM accounts
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
            WHERE guid = $1
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_commodity_guid_question_mark<DB, O, T>(
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
            WHERE commodity_guid = ?
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_commodity_guid_money_mark<DB, O, T>(
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
            WHERE commodity_guid = $1
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_parent_guid_question_mark<DB, O, T>(
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
            WHERE parent_guid = ?
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_parent_guid_money_mark<DB, O, T>(
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
            WHERE parent_guid = $1
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_name_question_mark<DB, O, T>(
        name: T,
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
            WHERE name = ?
            "#,
        )
        .bind(name)
    }

    pub(crate) fn query_by_name_money_mark<DB, O, T>(
        name: T,
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
            WHERE name = $1
            "#,
        )
        .bind(name)
    }

    pub(crate) fn query_like_name_question_mark<DB, O, T>(
        name: T,
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
            WHERE name LIKE ?
            "#,
        )
        .bind(name)
    }

    pub(crate) fn query_like_name_money_mark<DB, O, T>(
        name: T,
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
            WHERE LOWER(name) LIKE LOWER($1)
            "#,
        )
        .bind(name)
    }
}

#[cfg(test)]
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
            let result: Vec<Account> =
                block_on(async { Account::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(21, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Account = block_on(async {
                Account::query_by_guid_question_mark("fcd795021c976ba75621ec39e75f6214")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Asset", result.name);
        }

        #[test]
        fn query_by_commodity_guid() {
            let pool = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_commodity_guid_question_mark("346629655191dcf59a7e2c2a85b70f69")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(14, result.len());
        }

        #[test]
        fn query_by_parent_guid() {
            let pool = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_parent_guid_question_mark("fcd795021c976ba75621ec39e75f6214")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_name() {
            let pool = setup(URI);
            let result: Account = block_on(async {
                Account::query_by_name_question_mark("Asset")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("fcd795021c976ba75621ec39e75f6214", result.guid);
        }

        #[test]
        fn query_like_name() {
            let pool = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_like_name_question_mark("%AS%")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
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
            let result: Vec<Account> =
                block_on(async { Account::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(21, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Account = block_on(async {
                Account::query_by_guid_money_mark("fcd795021c976ba75621ec39e75f6214")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Asset", result.name);
        }

        #[test]
        fn query_by_commodity_guid() {
            let pool = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_commodity_guid_money_mark("346629655191dcf59a7e2c2a85b70f69")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(14, result.len());
        }

        #[test]
        fn query_by_parent_guid() {
            let pool = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_parent_guid_money_mark("fcd795021c976ba75621ec39e75f6214")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_name() {
            let pool = setup(URI);
            let result: Account = block_on(async {
                Account::query_by_name_money_mark("Asset")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("fcd795021c976ba75621ec39e75f6214", result.guid);
        }

        #[test]
        fn query_like_name() {
            let pool = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_like_name_money_mark("%AS%")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
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
            let result: Vec<Account> =
                block_on(async { Account::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(21, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Account = block_on(async {
                Account::query_by_guid_question_mark("fcd795021c976ba75621ec39e75f6214")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Asset", result.name);
        }

        #[test]
        fn query_by_commodity_guid() {
            let pool = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_commodity_guid_question_mark("346629655191dcf59a7e2c2a85b70f69")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(14, result.len());
        }

        #[test]
        fn query_by_parent_guid() {
            let pool = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_parent_guid_question_mark("fcd795021c976ba75621ec39e75f6214")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_name() {
            let pool = setup(URI);
            let result: Account = block_on(async {
                Account::query_by_name_question_mark("Asset")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("fcd795021c976ba75621ec39e75f6214", result.guid);
        }

        #[test]
        fn query_like_name() {
            let pool = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_like_name_question_mark("%AS%")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }
    }
}
