use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Price {
    pub guid: String,
    pub commodity_guid: String,
    pub currency_guid: String,
    pub date: chrono::NaiveDateTime,
    pub source: Option<String>,
    pub r#type: Option<String>,
    pub value_num: i64,
    pub value_denom: i64,
    pub value: f64,
}

impl<'q> Price {
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
            commodity_guid,
            currency_guid,
            date as "date: chrono::NaiveDateTime",
            source,
            type,
            value_num,
            value_denom,
            CAST(value_num AS float) / CAST(value_denom AS float) as "value!: f64"
            FROM prices
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
            commodity_guid,
            currency_guid,
            date,
            source,
            type,
            value_num,
            value_denom,
            CAST(value_num AS float) / CAST(value_denom AS float) as "value"
            FROM prices
            WHERE guid = ?
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_commodity_guid(
        guid: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT
            guid,
            commodity_guid,
            currency_guid,
            date,
            source,
            type,
            value_num,
            value_denom,
            CAST(value_num AS float) / CAST(value_denom AS float) as "value"
            FROM prices
            WHERE commodity_guid = ?
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
            commodity_guid,
            currency_guid,
            date,
            source,
            type,
            value_num,
            value_denom,
            CAST(value_num AS float) / CAST(value_denom AS float) as "value"
            FROM prices
            WHERE currency_guid = ?
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_commodity_or_currency_guid(
        guid: &'q str,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, Self, sqlx::sqlite::SqliteArguments<'q>> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT
            guid,
            commodity_guid,
            currency_guid,
            date,
            source,
            type,
            value_num,
            value_denom,
            CAST(value_num AS float) / CAST(value_denom AS float) as "value"
            FROM prices
            WHERE commodity_guid = ?
            OR currency_guid = ?
            "#,
        )
        .bind(guid)
        .bind(guid)
    }

    pub fn value(&self) -> Decimal {
        Decimal::from_str(&self.value_num.to_string()).unwrap()
            / Decimal::from_str(&self.value_denom.to_string()).unwrap()
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
            let result = block_on(async { Price::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result = block_on(async {
                Price::query_by_guid("0d6684f44fb018e882de76094ed9c433")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_guid() {
            let pool = setup(URI);
            let result = block_on(async {
                Price::query_by_commodity_guid("d821d6776fde9f7c2d01b67876406fd3")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_currency_guid() {
            let pool = setup(URI);
            let result = block_on(async {
                Price::query_by_currency_guid("5f586908098232e67edb1371408bfaa8")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_or_currency_guid() {
            let pool = setup(URI);
            let result = block_on(async {
                Price::query_by_commodity_or_currency_guid("5f586908098232e67edb1371408bfaa8")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(4, result.len());
        }
    }
}
