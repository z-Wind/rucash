#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
use super::TestSchemas;
use rust_decimal::Decimal;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::kind::SQLKind;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
#[cfg_attr(
    any(feature = "sqlite", feature = "postgres", feature = "mysql",),
    derive(sqlx::FromRow)
)]
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
    pub quantity_num: i64,
    pub quantity_denom: i64,
    pub lot_guid: Option<String>,
}

impl super::NullNone for Split {
    fn null_none(self) -> Self {
        let lot_guid = self.lot_guid.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });

        let reconcile_date = self.reconcile_date.and_then(|x| {
            match x.format("%Y-%m-%d %H:%M:%S").to_string().as_str() {
                "1970-01-01 00:00:00" => None,
                _ => self.reconcile_date,
            }
        });

        Self {
            lot_guid,
            reconcile_date,
            ..self
        }
    }
}

impl<'q> Split {
    // test schemas on compile time
    #[allow(dead_code)]
    #[cfg(feature = "sqlite")]
    fn test_schemas() -> TestSchemas<
        'q,
        sqlx::Sqlite,
        sqlx::sqlite::SqliteRow,
        Self,
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
            quantity_num,
            quantity_denom,
            lot_guid
            FROM splits
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
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date,
            value_num,
            value_denom,
            quantity_num,
            quantity_denom,
            lot_guid
            FROM splits
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
                tx_guid,
                account_guid,
                memo,
                action,
                reconcile_state,
                reconcile_date,
                value_num,
                value_denom,
                quantity_num,
                quantity_denom,
                lot_guid
                FROM splits
                WHERE guid = $1
                "#,
            )
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            quantity_num,
            quantity_denom,
            lot_guid
            FROM splits
            WHERE guid = ?
            "#,
            )
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
    pub(crate) fn query_by_account_guid<DB, O, T>(
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
                tx_guid,
                account_guid,
                memo,
                action,
                reconcile_state,
                reconcile_date,
                value_num,
                value_denom,
                quantity_num,
                quantity_denom,
                lot_guid
                FROM splits
                WHERE account_guid = $1
                "#,
            )
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            quantity_num,
            quantity_denom,
            lot_guid
            FROM splits
            WHERE account_guid = ?
            "#,
            )
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
    pub(crate) fn query_by_tx_guid<DB, O, T>(
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
                tx_guid,
                account_guid,
                memo,
                action,
                reconcile_state,
                reconcile_date,
                value_num,
                value_denom,
                quantity_num,
                quantity_denom,
                lot_guid
                FROM splits
                WHERE tx_guid = $1
                "#,
            )
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            quantity_num,
            quantity_denom,
            lot_guid
            FROM splits
            WHERE tx_guid = ?
            "#,
            )
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }

    pub fn value(&self) -> f64 {
        self.value_num as f64 / self.value_denom as f64
    }

    pub fn value_into_decimal(&self) -> Decimal {
        Decimal::new(self.value_num, 0) / Decimal::new(self.value_denom, 0)
    }

    pub fn quantity(&self) -> f64 {
        self.quantity_num as f64 / self.quantity_denom as f64
    }

    pub fn quantity_into_decimal(&self) -> Decimal {
        Decimal::new(self.quantity_num, 0) / Decimal::new(self.quantity_denom, 0)
    }
}

#[cfg(feature = "xml")]
use xmltree::Element;
#[cfg(feature = "xml")]
impl Split {
    pub(crate) fn new_by_element(tx_guid: String, e: &Element) -> Self {
        let guid = e
            .get_child("id")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("id must exist");
        let tx_guid = tx_guid;
        let account_guid = e
            .get_child("account")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("account must exist");
        let memo = e
            .get_child("memo")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .unwrap_or_default();
        let action = e
            .get_child("action")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .unwrap_or_default();
        let reconcile_state = e
            .get_child("reconciled-state")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .unwrap_or_default();
        let reconcile_date = e
            .get_child("reconciled-date")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S")
                    .expect("%Y-%m-%d %H:%M:%S")
            });

        let splits = e
            .get_child("value")
            .expect("value must exist")
            .get_text()
            .unwrap();
        let mut splits = splits.split('/');
        let value_num = splits.next().unwrap().parse().unwrap();
        let value_denom = splits.next().unwrap().parse().unwrap();

        let splits = e
            .get_child("quantity")
            .expect("quantity must exist")
            .get_text()
            .unwrap();
        let mut splits = splits.split('/');
        let quantity_num = splits.next().unwrap().parse().unwrap();
        let quantity_denom = splits.next().unwrap().parse().unwrap();
        let lot_guid = None;

        Self {
            guid,
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date,
            value_num,
            value_denom,
            quantity_num,
            quantity_denom,
            lot_guid,
        }
    }
}

#[cfg(test)]
#[cfg(any(
    feature = "sqlite",
    feature = "postgres",
    feature = "mysql",
    feature = "xml"
))]
mod tests {
    use super::*;
    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
    use futures::executor::block_on;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        type DB = sqlx::Sqlite;

        fn setup() -> (sqlx::Pool<DB>, SQLKind) {
            let uri: &str = &format!(
                "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );
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
            let (pool, _kind) = setup();
            let result: Vec<Split> =
                block_on(async { Split::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(25, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup();
            let result: Split = block_on(async {
                Split::query_by_guid("de832fe97e37811a7fff7e28b3a43425", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(150.0, result.value());
            assert_eq!(Decimal::new(150, 0), result.value_into_decimal());
        }

        #[test]
        fn query_by_account_guid() {
            let (pool, kind) = setup();
            let result: Vec<Split> = block_on(async {
                Split::query_by_account_guid("93fc043c3062aaa1297b30e543d2cd0d", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_tx_guid() {
            let (pool, kind) = setup();
            let result: Vec<Split> = block_on(async {
                Split::query_by_tx_guid("6c8876003c4a6026e38e3afb67d6f2b1", kind)
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

        type DB = sqlx::Postgres;

        fn setup() -> (sqlx::Pool<DB>, SQLKind) {
            let uri: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
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
            let (pool, _kind) = setup();
            let result: Vec<Split> =
                block_on(async { Split::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(25, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup();
            let result: Split = block_on(async {
                Split::query_by_guid("de832fe97e37811a7fff7e28b3a43425", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(150.0, result.value());
            assert_eq!(Decimal::new(150, 0), result.value_into_decimal());
        }

        #[test]
        fn query_by_account_guid() {
            let (pool, kind) = setup();
            let result: Vec<Split> = block_on(async {
                Split::query_by_account_guid("93fc043c3062aaa1297b30e543d2cd0d", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_tx_guid() {
            let (pool, kind) = setup();
            let result: Vec<Split> = block_on(async {
                Split::query_by_tx_guid("6c8876003c4a6026e38e3afb67d6f2b1", kind)
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

        type DB = sqlx::MySql;

        fn setup() -> (sqlx::Pool<DB>, SQLKind) {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
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
            let (pool, _kind) = setup();
            let result: Vec<Split> =
                block_on(async { Split::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(25, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup();
            let result: Split = block_on(async {
                Split::query_by_guid("de832fe97e37811a7fff7e28b3a43425", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(150.0, result.value());
            assert_eq!(Decimal::new(150, 0), result.value_into_decimal());
        }

        #[test]
        fn query_by_account_guid() {
            let (pool, kind) = setup();
            let result: Vec<Split> = block_on(async {
                Split::query_by_account_guid("93fc043c3062aaa1297b30e543d2cd0d", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_tx_guid() {
            let (pool, kind) = setup();
            let result: Vec<Split> = block_on(async {
                Split::query_by_tx_guid("6c8876003c4a6026e38e3afb67d6f2b1", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(2, result.len());
        }
    }

    #[cfg(feature = "xml")]
    mod xml {
        use super::*;
        use std::sync::Arc;

        #[allow(dead_code)]
        fn setup() -> Arc<Element> {
            let uri: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );
            crate::XMLBook::new(uri).unwrap().pool.0.clone()
        }

        #[test]
        fn new_by_element() {
            let data = r##"
                <?xml version="1.0" encoding="utf-8" ?>
                <gnc-v2
                    xmlns:gnc="http://www.gnucash.org/XML/gnc"
                    xmlns:act="http://www.gnucash.org/XML/act"
                    xmlns:book="http://www.gnucash.org/XML/book"
                    xmlns:cd="http://www.gnucash.org/XML/cd"
                    xmlns:cmdty="http://www.gnucash.org/XML/cmdty"
                    xmlns:price="http://www.gnucash.org/XML/price"
                    xmlns:slot="http://www.gnucash.org/XML/slot"
                    xmlns:split="http://www.gnucash.org/XML/split"
                    xmlns:sx="http://www.gnucash.org/XML/sx"
                    xmlns:trn="http://www.gnucash.org/XML/trn"
                    xmlns:ts="http://www.gnucash.org/XML/ts"
                    xmlns:fs="http://www.gnucash.org/XML/fs"
                    xmlns:bgt="http://www.gnucash.org/XML/bgt"
                    xmlns:recurrence="http://www.gnucash.org/XML/recurrence"
                    xmlns:lot="http://www.gnucash.org/XML/lot"
                    xmlns:addr="http://www.gnucash.org/XML/addr"
                    xmlns:billterm="http://www.gnucash.org/XML/billterm"
                    xmlns:bt-days="http://www.gnucash.org/XML/bt-days"
                    xmlns:bt-prox="http://www.gnucash.org/XML/bt-prox"
                    xmlns:cust="http://www.gnucash.org/XML/cust"
                    xmlns:employee="http://www.gnucash.org/XML/employee"
                    xmlns:entry="http://www.gnucash.org/XML/entry"
                    xmlns:invoice="http://www.gnucash.org/XML/invoice"
                    xmlns:job="http://www.gnucash.org/XML/job"
                    xmlns:order="http://www.gnucash.org/XML/order"
                    xmlns:owner="http://www.gnucash.org/XML/owner"
                    xmlns:taxtable="http://www.gnucash.org/XML/taxtable"
                    xmlns:tte="http://www.gnucash.org/XML/tte"
                    xmlns:vendor="http://www.gnucash.org/XML/vendor">
                    <trn:split>
                        <split:id type="guid">de832fe97e37811a7fff7e28b3a43425</split:id>
                        <split:reconciled-state>n</split:reconciled-state>
                        <split:value>15000/100</split:value>
                        <split:quantity>15000/100</split:quantity>
                        <split:account type="guid">93fc043c3062aaa1297b30e543d2cd0d</split:account>
                    </trn:split>
                </gnc-v2>
                "##;

            let e = Element::parse(data.as_bytes())
                .unwrap()
                .take_child("split")
                .unwrap();

            let split = Split::new_by_element(String::from("6c8876003c4a6026e38e3afb67d6f2b1"), &e);

            assert_eq!(split.guid, "de832fe97e37811a7fff7e28b3a43425");
            assert_eq!(split.tx_guid, "6c8876003c4a6026e38e3afb67d6f2b1");
            assert_eq!(split.account_guid, "93fc043c3062aaa1297b30e543d2cd0d");
            assert_eq!(split.memo, "");
            assert_eq!(split.action, "");
            assert_eq!(split.reconcile_state, "n");
            assert_eq!(split.reconcile_date, None);
            assert_eq!(split.value_num, 15000);
            assert_eq!(split.value_denom, 100);
            assert_eq!(split.value(), 150.0);
            assert_eq!(split.quantity_num, 15000);
            assert_eq!(split.quantity_denom, 100);
            assert_eq!(split.quantity(), 150.0);
            assert_eq!(split.lot_guid, None);
        }
    }
}
