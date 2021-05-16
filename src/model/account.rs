#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Account {
    pub guid: String,
    pub name: String,
    pub account_type: String,
    pub commodity_guid: Option<String>,
    pub commodity_scu: i64,
    pub non_std_scu: i64,
    pub parent_guid: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub hidden: Option<bool>,
    pub placeholder: Option<bool>,
}

impl<'q> Account {
    pub fn query() -> sqlx::query::Map<
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
            commodity_scu,
            non_std_scu,
            parent_guid,
            code,
            description,
            hidden as "hidden: bool",
            placeholder as "placeholder: bool"
            FROM accounts
            "#,
        )
    }

    pub fn query_by_guid(
        guid: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query_as::<_, Self>(
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

    pub fn query_by_commodity_guid(
        guid: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query_as::<_, Self>(
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

    pub fn query_by_parent_guid(
        guid: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query_as::<_, Self>(
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

    pub fn query_by_name(
        name: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query_as::<_, Self>(
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

    pub fn query_like_name(
        name: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query_as::<_, Self>(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    mod sqlite {
        use super::*;

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
            let pool = setup("sqlite://tests/sample/complex_sample.gnucash");
            let result = block_on(async { Account::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(21, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup("sqlite://tests/sample/complex_sample.gnucash");
            let result = block_on(async {
                Account::query_by_guid("fcd795021c976ba75621ec39e75f6214")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Asset", result.name);
        }

        #[test]
        fn query_by_commodity_guid() {
            let pool = setup("sqlite://tests/sample/complex_sample.gnucash");
            let result = block_on(async {
                Account::query_by_commodity_guid("346629655191dcf59a7e2c2a85b70f69")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(14, result.len());
        }

        #[test]
        fn query_by_parent_guid() {
            let pool = setup("sqlite://tests/sample/complex_sample.gnucash");
            let result = block_on(async {
                Account::query_by_parent_guid("fcd795021c976ba75621ec39e75f6214")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_name() {
            let pool = setup("sqlite://tests/sample/complex_sample.gnucash");
            let result =
                block_on(async { Account::query_by_name("Asset").fetch_one(&pool).await }).unwrap();
            assert_eq!("fcd795021c976ba75621ec39e75f6214", result.guid);
        }

        #[test]
        fn query_like_name() {
            let pool = setup("sqlite://tests/sample/complex_sample.gnucash");
            let result =
                block_on(async { Account::query_like_name("%AS%").fetch_all(&pool).await })
                    .unwrap();
            assert_eq!(3, result.len());
        }
    }
}
