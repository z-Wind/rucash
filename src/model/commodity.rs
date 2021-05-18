#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Commodity {
    pub guid: String,
    pub namespace: String,
    pub mnemonic: String,
    pub fullname: Option<String>,
    pub cusip: Option<String>,
    pub fraction: i64,
    pub quote_flag: i64,
    pub quote_source: Option<String>,
    pub quote_tz: Option<String>,
}

impl<'q> Commodity {
    pub(crate) fn query() -> sqlx::query::Map<
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
            fraction,
            quote_flag,
            quote_source,
            quote_tz
            FROM commodities
            "#,
        )
    }

    pub(crate) fn query_by_guid(
        guid: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query_as::<_, Self>(
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

    pub(crate) fn query_by_namespace(
        namespace: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query_as::<_, Self>(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    const URI: &str = "sqlite://tests/sqlite/sample/complex_sample.gnucash";
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
            let pool = setup(URI);
            let result = block_on(async { Commodity::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result = block_on(async {
                Commodity::query_by_guid("346629655191dcf59a7e2c2a85b70f69")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Euro", result.fullname.unwrap());
        }

        #[test]
        fn query_by_namespace() {
            let pool = setup(URI);
            let result = block_on(async {
                Commodity::query_by_namespace("CURRENCY")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(4, result.len());
        }
    }
}
