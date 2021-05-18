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
            currency_guid,
            num,
            post_date as "post_date: chrono::NaiveDateTime",
            enter_date as "enter_date: chrono::NaiveDateTime",
            description
            FROM transactions
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

    pub(crate) fn query_by_currency_guid(
        guid: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query_as::<_, Self>(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    const URI: &str = "sqlite://tests/sqlite/sample/complex_sample.gnucash";
    mod sqlite {
        use chrono::NaiveDateTime;

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
            let result = block_on(async { Transaction::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(11, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result = block_on(async {
                Transaction::query_by_guid("6c8876003c4a6026e38e3afb67d6f2b1")
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
            let result = block_on(async {
                Transaction::query_by_currency_guid("346629655191dcf59a7e2c2a85b70f69")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(11, result.len());
        }
    }
}
