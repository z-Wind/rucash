use rust_decimal::Decimal;
use std::str::FromStr;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::kind::SQLKind;
#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(
    any(feature = "sqlite", feature = "postgres", feature = "mysql",),
    derive(sqlx::FromRow)
)]
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

impl super::NullNone for Price {
    fn null_none(self) -> Self {
        let source = self.source.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });
        let r#type = self.r#type.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });

        Self {
            source,
            r#type,
            ..self
        }
    }
}

impl<'q> Price {
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
            commodity_guid,
            currency_guid,
            date,
            source,
            type,
            value_num,
            value_denom,
            CAST(value_num AS float) / CAST(value_denom AS float) as value
            FROM prices
            "#,
        )
    }

    #[allow(dead_code)]
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
                commodity_guid,
                currency_guid,
                date,
                source,
                type,
                value_num,
                value_denom,
                CAST(value_num AS float) / CAST(value_denom AS float) as "value"
                FROM prices
                WHERE guid = $1
                "#,
            )
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
    pub(crate) fn query_by_commodity_guid<DB, O, T>(
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
                commodity_guid,
                currency_guid,
                date,
                source,
                type,
                value_num,
                value_denom,
                CAST(value_num AS float) / CAST(value_denom AS float) as "value"
                FROM prices
                WHERE commodity_guid = $1
                "#,
            )
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
                commodity_guid,
                currency_guid,
                date,
                source,
                type,
                value_num,
                value_denom,
                CAST(value_num AS float) / CAST(value_denom AS float) as "value"
                FROM prices
                WHERE currency_guid = $1
                "#,
            )
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
    pub(crate) fn query_by_commodity_or_currency_guid<DB, O, T>(
        guid: T,
        kind: SQLKind,
    ) -> sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::HasArguments<'q>>::Arguments>
    where
        DB: sqlx::Database,
        O: Send + Unpin + for<'r> sqlx::FromRow<'r, DB::Row>,
        T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB> + Clone,
    {
        match kind {
            SQLKind::Postgres => sqlx::query_as(
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
                WHERE commodity_guid = $1
                OR currency_guid = $1
                "#,
            )
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            .bind(guid.clone())
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }

    pub fn value(&self) -> Decimal {
        Decimal::from_str(&self.value_num.to_string()).unwrap()
            / Decimal::from_str(&self.value_denom.to_string()).unwrap()
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
            let result: Vec<Price> =
                block_on(async { Price::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup(URI);
            let result: Price = block_on(async {
                Price::query_by_guid("0d6684f44fb018e882de76094ed9c433", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_guid() {
            let (pool, kind) = setup(URI);
            let result: Price = block_on(async {
                Price::query_by_commodity_guid("d821d6776fde9f7c2d01b67876406fd3", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_currency_guid() {
            let (pool, kind) = setup(URI);
            let result: Price = block_on(async {
                Price::query_by_currency_guid("5f586908098232e67edb1371408bfaa8", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_or_currency_guid() {
            let (pool, kind) = setup(URI);
            let result: Vec<Price> = block_on(async {
                Price::query_by_commodity_or_currency_guid("5f586908098232e67edb1371408bfaa8", kind)
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
            let result: Vec<Price> =
                block_on(async { Price::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup(URI);
            let result: Price = block_on(async {
                Price::query_by_guid("0d6684f44fb018e882de76094ed9c433", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_guid() {
            let (pool, kind) = setup(URI);
            let result: Price = block_on(async {
                Price::query_by_commodity_guid("d821d6776fde9f7c2d01b67876406fd3", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_currency_guid() {
            let (pool, kind) = setup(URI);
            let result: Price = block_on(async {
                Price::query_by_currency_guid("5f586908098232e67edb1371408bfaa8", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_or_currency_guid() {
            let (pool, kind) = setup(URI);
            let result: Vec<Price> = block_on(async {
                Price::query_by_commodity_or_currency_guid("5f586908098232e67edb1371408bfaa8", kind)
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
            let result: Vec<Price> =
                block_on(async { Price::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup(URI);
            let result: Price = block_on(async {
                Price::query_by_guid("0d6684f44fb018e882de76094ed9c433", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_guid() {
            let (pool, kind) = setup(URI);
            let result: Price = block_on(async {
                Price::query_by_commodity_guid("d821d6776fde9f7c2d01b67876406fd3", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_currency_guid() {
            let (pool, kind) = setup(URI);
            let result: Price = block_on(async {
                Price::query_by_currency_guid("5f586908098232e67edb1371408bfaa8", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(1.5, result.value);
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_or_currency_guid() {
            let (pool, kind) = setup(URI);
            let result: Vec<Price> = block_on(async {
                Price::query_by_commodity_or_currency_guid("5f586908098232e67edb1371408bfaa8", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(4, result.len());
        }
    }
}
