use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Clone, Debug)]
#[cfg_attr(any(
    feature = "sqlite",
    feature = "postgres",
    feature = "mysql",
), derive(sqlx::FromRow))]
pub struct Split {
    pub guid: String,
    pub tx_guid: String,
    pub account_guid: String,
    pub memo: String,
    pub action: String,
    pub reconcile_state: String,
    pub reconcile_date: Option<chrono::NaiveDateTime>,
    pub value_num: i64,
    pub value_denom: i64,
    pub value: f64,
    pub quantity_num: i64,
    pub quantity_denom: i64,
    pub quantity: f64,
    pub lot_guid: Option<String>,
}

#[cfg(any(
    feature = "sqlite",
    feature = "postgres",
    feature = "mysql",
))]
impl<'q> Split {
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
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date as "reconcile_date: chrono::NaiveDateTime",
            value_num,
            value_denom,
            CAST(value_num AS float)/ CAST(value_denom AS float) as "value!: f64",
            quantity_num,
            quantity_denom,
            CAST(quantity_num AS float) / CAST(quantity_denom AS float) as "quantity!: f64",
            lot_guid
            FROM splits
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
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date,
            value_num,
            value_denom,
            CAST(value_num AS float)/ CAST(value_denom AS float) as value,
            quantity_num,
            quantity_denom,
            CAST(quantity_num AS float) / CAST(quantity_denom AS float) as quantity,
            lot_guid
            FROM splits
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
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date,
            value_num,
            value_denom,
            CAST(value_num AS float)/ CAST(value_denom AS float) as "value",
            quantity_num,
            quantity_denom,
            CAST(quantity_num AS float) / CAST(quantity_denom AS float) as "quantity",
            lot_guid
            FROM splits
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
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date,
            value_num,
            value_denom,
            CAST(value_num AS float)/ CAST(value_denom AS float) as "value",
            quantity_num,
            quantity_denom,
            CAST(quantity_num AS float) / CAST(quantity_denom AS float) as "quantity",
            lot_guid
            FROM splits
            WHERE guid = $1
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_account_guid_question_mark<DB, O, T>(
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
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date,
            value_num,
            value_denom,
            CAST(value_num AS float)/ CAST(value_denom AS float) as "value" ,
            quantity_num,
            quantity_denom,
            CAST(quantity_num AS float) / CAST(quantity_denom AS float) as "quantity",
            lot_guid
            FROM splits
            WHERE account_guid = ?
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_account_guid_money_mark<DB, O, T>(
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
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date,
            value_num,
            value_denom,
            CAST(value_num AS float)/ CAST(value_denom AS float) as "value" ,
            quantity_num,
            quantity_denom,
            CAST(quantity_num AS float) / CAST(quantity_denom AS float) as "quantity",
            lot_guid
            FROM splits
            WHERE account_guid = $1
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_tx_guid_question_mark<DB, O, T>(
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
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date,
            value_num,
            value_denom,
            CAST(value_num AS float)/ CAST(value_denom AS float) as "value" ,
            quantity_num,
            quantity_denom,
            CAST(quantity_num AS float) / CAST(quantity_denom AS float) as "quantity",
            lot_guid
            FROM splits
            WHERE tx_guid = ?
            "#,
        )
        .bind(guid)
    }

    pub(crate) fn query_by_tx_guid_money_mark<DB, O, T>(
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
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date,
            value_num,
            value_denom,
            CAST(value_num AS float)/ CAST(value_denom AS float) as "value" ,
            quantity_num,
            quantity_denom,
            CAST(quantity_num AS float) / CAST(quantity_denom AS float) as "quantity",
            lot_guid
            FROM splits
            WHERE tx_guid = $1
            "#,
        )
        .bind(guid)
    }

    pub fn value(&self) -> Decimal {
        Decimal::from_str(&self.value_num.to_string()).unwrap()
            / Decimal::from_str(&self.value_denom.to_string()).unwrap()
    }

    pub fn quantity(&self) -> Decimal {
        Decimal::from_str(&self.quantity_num.to_string()).unwrap()
            / Decimal::from_str(&self.quantity_denom.to_string()).unwrap()
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
            let result: Vec<Split> =
                block_on(async { Split::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(25, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Split = block_on(async {
                Split::query_by_guid_question_mark("de832fe97e37811a7fff7e28b3a43425")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(150.0, result.value);
            assert_eq!(Decimal::new(150, 0), result.value());
        }

        #[test]
        fn query_by_account_guid() {
            let pool = setup(URI);
            let result: Vec<Split> = block_on(async {
                Split::query_by_account_guid_question_mark("93fc043c3062aaa1297b30e543d2cd0d")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_tx_guid() {
            let pool = setup(URI);
            let result: Vec<Split> = block_on(async {
                Split::query_by_tx_guid_question_mark("6c8876003c4a6026e38e3afb67d6f2b1")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(2, result.len());
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
            let result: Vec<Split> =
                block_on(async { Split::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(25, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Split = block_on(async {
                Split::query_by_guid_money_mark("de832fe97e37811a7fff7e28b3a43425")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(150.0, result.value);
            assert_eq!(Decimal::new(150, 0), result.value());
        }

        #[test]
        fn query_by_account_guid() {
            let pool = setup(URI);
            let result: Vec<Split> = block_on(async {
                Split::query_by_account_guid_money_mark("93fc043c3062aaa1297b30e543d2cd0d")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_tx_guid() {
            let pool = setup(URI);
            let result: Vec<Split> = block_on(async {
                Split::query_by_tx_guid_money_mark("6c8876003c4a6026e38e3afb67d6f2b1")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(2, result.len());
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
            let result: Vec<Split> =
                block_on(async { Split::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(25, result.len());
        }

        #[test]
        fn query_by_guid() {
            let pool = setup(URI);
            let result: Split = block_on(async {
                Split::query_by_guid_question_mark("de832fe97e37811a7fff7e28b3a43425")
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(150.0, result.value);
            assert_eq!(Decimal::new(150, 0), result.value());
        }

        #[test]
        fn query_by_account_guid() {
            let pool = setup(URI);
            let result: Vec<Split> = block_on(async {
                Split::query_by_account_guid_question_mark("93fc043c3062aaa1297b30e543d2cd0d")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_tx_guid() {
            let pool = setup(URI);
            let result: Vec<Split> = block_on(async {
                Split::query_by_tx_guid_question_mark("6c8876003c4a6026e38e3afb67d6f2b1")
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(2, result.len());
        }
    }
}
