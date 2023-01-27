#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
use super::TestSchemas;
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::kind::SQLKind;
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
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
            commodity_guid,
            currency_guid,
            date as "date: chrono::NaiveDateTime",
            source,
            type,
            value_num,
            value_denom
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
            value_denom
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
                value_denom
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
            value_denom
            FROM prices
            WHERE guid = ?
            "#,
            )
            .bind(guid),
            _ => panic!("{kind:?} not support"),
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
                value_denom
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
            value_denom
            FROM prices
            WHERE commodity_guid = ?
            "#,
            )
            .bind(guid),
            _ => panic!("{kind:?} not support"),
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
                value_denom
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
            value_denom
            FROM prices
            WHERE currency_guid = ?
            "#,
            )
            .bind(guid),
            _ => panic!("{kind:?} not support"),
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
                value_denom
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
            value_denom
            FROM prices
            WHERE commodity_guid = ?
            OR currency_guid = ?
            "#,
            )
            .bind(guid.clone())
            .bind(guid),
            _ => panic!("{kind:?} not support"),
        }
    }

    #[cfg(not(feature = "decimal"))]
    pub fn value(&self) -> f64 {
        self.value_num as f64 / self.value_denom as f64
    }

    #[cfg(feature = "decimal")]
    pub fn value(&self) -> Decimal {
        Decimal::new(self.value_num, 0) / Decimal::new(self.value_denom, 0)
    }
}

#[cfg(feature = "xml")]
use xmltree::Element;
#[cfg(feature = "xml")]

impl Price {
    pub(crate) fn new_by_element(e: &Element) -> Self {
        let guid = e
            .get_child("id")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("id must exist");
        let commodity_guid = e
            .get_child("commodity")
            .and_then(|x| x.get_child("id"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("commodity must exist");
        let currency_guid = e
            .get_child("currency")
            .and_then(|x| x.get_child("id"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("currency must exist");
        let date = e
            .get_child("time")
            .and_then(|x| x.get_child("date"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z")
                    .expect("%Y-%m-%d %H:%M:%S%z")
            })
            .expect("time must exist");
        let source = e
            .get_child("source")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());

        let r#type = e
            .get_child("type")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());

        let splits = e
            .get_child("value")
            .expect("value must exist")
            .get_text()
            .unwrap();
        let mut splits = splits.split('/');
        let value_num = splits.next().unwrap().parse().unwrap();
        let value_denom = splits.next().unwrap().parse().unwrap();

        Self {
            guid,
            commodity_guid,
            currency_guid,
            date,
            source,
            r#type,
            value_num,
            value_denom,
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
    use tokio::runtime::Runtime;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        type DB = sqlx::Sqlite;

        fn setup() -> (sqlx::Pool<DB>, Runtime, SQLKind) {
            let uri: &str = &format!(
                "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            (
                rt.block_on(async {
                    sqlx::sqlite::SqlitePoolOptions::new()
                        .max_connections(5)
                        .connect(&format!("{uri}?mode=ro")) // read only
                        .await
                        .unwrap()
                }),
                rt,
                uri.parse().expect("sqlite"),
            )
        }

        #[test]
        fn query() {
            let (pool, rt, _) = setup();
            let result: Vec<Price> = rt
                .block_on(async { Price::query().fetch_all(&pool).await })
                .unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, rt, kind) = setup();
            let result: Price = rt
                .block_on(async {
                    Price::query_by_guid("0d6684f44fb018e882de76094ed9c433", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_eq!(1.5, result.value());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_guid() {
            let (pool, rt, kind) = setup();
            let result: Price = rt
                .block_on(async {
                    Price::query_by_commodity_guid("d821d6776fde9f7c2d01b67876406fd3", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_eq!(1.5, result.value());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_currency_guid() {
            let (pool, rt, kind) = setup();
            let result: Price = rt
                .block_on(async {
                    Price::query_by_currency_guid("5f586908098232e67edb1371408bfaa8", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_eq!(1.5, result.value());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_or_currency_guid() {
            let (pool, rt, kind) = setup();
            let result: Vec<Price> = rt
                .block_on(async {
                    Price::query_by_commodity_or_currency_guid(
                        "5f586908098232e67edb1371408bfaa8",
                        kind,
                    )
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

        type DB = sqlx::Postgres;

        fn setup() -> (sqlx::Pool<DB>, Runtime, SQLKind) {
            let uri: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            (
                rt.block_on(async {
                    sqlx::postgres::PgPoolOptions::new()
                        .max_connections(5)
                        .connect(uri)
                        .await
                        .unwrap()
                }),
                rt,
                uri.parse().expect("postgres"),
            )
        }

        #[test]
        fn query() {
            let (pool, rt, _) = setup();
            let result: Vec<Price> = rt
                .block_on(async { Price::query().fetch_all(&pool).await })
                .unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, rt, kind) = setup();
            let result: Price = rt
                .block_on(async {
                    Price::query_by_guid("0d6684f44fb018e882de76094ed9c433", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_eq!(1.5, result.value());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_guid() {
            let (pool, rt, kind) = setup();
            let result: Price = rt
                .block_on(async {
                    Price::query_by_commodity_guid("d821d6776fde9f7c2d01b67876406fd3", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_eq!(1.5, result.value());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_currency_guid() {
            let (pool, rt, kind) = setup();
            let result: Price = rt
                .block_on(async {
                    Price::query_by_currency_guid("5f586908098232e67edb1371408bfaa8", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_eq!(1.5, result.value());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_or_currency_guid() {
            let (pool, rt, kind) = setup();
            let result: Vec<Price> = rt
                .block_on(async {
                    Price::query_by_commodity_or_currency_guid(
                        "5f586908098232e67edb1371408bfaa8",
                        kind,
                    )
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

        type DB = sqlx::MySql;

        fn setup() -> (sqlx::Pool<DB>, Runtime, SQLKind) {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            (
                rt.block_on(async {
                    sqlx::mysql::MySqlPoolOptions::new()
                        .max_connections(5)
                        .connect(uri)
                        .await
                        .unwrap()
                }),
                rt,
                uri.parse().expect("mysql"),
            )
        }

        #[test]
        fn query() {
            let (pool, rt, _) = setup();
            let result: Vec<Price> = rt
                .block_on(async { Price::query().fetch_all(&pool).await })
                .unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, rt, kind) = setup();
            let result: Price = rt
                .block_on(async {
                    Price::query_by_guid("0d6684f44fb018e882de76094ed9c433", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_eq!(1.5, result.value());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_guid() {
            let (pool, rt, kind) = setup();
            let result: Price = rt
                .block_on(async {
                    Price::query_by_commodity_guid("d821d6776fde9f7c2d01b67876406fd3", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_eq!(1.5, result.value());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_currency_guid() {
            let (pool, rt, kind) = setup();
            let result: Price = rt
                .block_on(async {
                    Price::query_by_currency_guid("5f586908098232e67edb1371408bfaa8", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_eq!(1.5, result.value());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), result.value());
        }

        #[test]
        fn query_by_commodity_or_currency_guid() {
            let (pool, rt, kind) = setup();
            let result: Vec<Price> = rt
                .block_on(async {
                    Price::query_by_commodity_or_currency_guid(
                        "5f586908098232e67edb1371408bfaa8",
                        kind,
                    )
                    .fetch_all(&pool)
                    .await
                })
                .unwrap();
            assert_eq!(4, result.len());
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
            crate::XMLBook::new(uri).unwrap().pool.0
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
                <price>
                    <price:id type="guid">0d6684f44fb018e882de76094ed9c433</price:id>
                    <price:commodity>
                        <cmdty:space>CURRENCY</cmdty:space>
                        <cmdty:id>ADF</cmdty:id>
                    </price:commodity>
                    <price:currency>
                        <cmdty:space>CURRENCY</cmdty:space>
                        <cmdty:id>AED</cmdty:id>
                    </price:currency>
                    <price:time>
                        <ts:date>2018-02-20 23:00:00 +0000</ts:date>
                    </price:time>
                    <price:source>user:price-editor</price:source>
                    <price:type>unknown</price:type>
                    <price:value>3/2</price:value>
                </price>
                </gnc-v2>
                "##;

            let e = Element::parse(data.as_bytes())
                .unwrap()
                .take_child("price")
                .unwrap();

            let price = Price::new_by_element(&e);

            assert_eq!(price.guid, "0d6684f44fb018e882de76094ed9c433");
            assert_eq!(price.commodity_guid, "ADF");
            assert_eq!(price.currency_guid, "AED");
            assert_eq!(
                price.date.format("%Y-%m-%d %H:%M:%S").to_string(),
                "2018-02-20 23:00:00"
            );
            assert_eq!(price.source.as_ref().unwrap(), "user:price-editor");
            assert_eq!(price.r#type.as_ref().unwrap(), "unknown");
            assert_eq!(price.value_num, 3);
            assert_eq!(price.value_denom, 2);
            #[cfg(not(feature = "decimal"))]
            assert_eq!(1.5, price.value());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), price.value());
        }
    }
}
