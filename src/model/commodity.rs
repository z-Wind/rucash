#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
use super::TestSchemas;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::kind::SQLKind;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
#[cfg_attr(
    any(feature = "sqlite", feature = "postgres", feature = "mysql",),
    derive(sqlx::FromRow)
)]
pub struct Commodity {
    pub guid: String,
    pub namespace: String,
    pub mnemonic: String,
    pub fullname: Option<String>,
    pub cusip: Option<String>,
    pub fraction: i32,
    pub quote_flag: i32,
    pub quote_source: Option<String>,
    pub quote_tz: Option<String>,
}

impl super::NullNone for Commodity {
    fn null_none(self) -> Self {
        let fullname = self.fullname.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });
        let cusip = self.cusip.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });
        let quote_source = self.quote_source.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });
        let quote_tz = self.quote_tz.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });

        Self {
            fullname,
            cusip,
            quote_source,
            quote_tz,
            ..self
        }
    }
}

impl<'q> Commodity {
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
            namespace,
            mnemonic,
            fullname,
            cusip,
            fraction as "fraction: i32",
            quote_flag as "quote_flag: i32",
            quote_source,
            quote_tz
            FROM commodities
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
                namespace,
                mnemonic,
                fullname,
                cusip,
                fraction,
                quote_flag,
                quote_source,
                quote_tz
                FROM commodities
                WHERE guid = $1
                "#,
            )
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            .bind(guid),
            _ => panic!("{kind:?} not support"),
        }
    }

    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
    pub(crate) fn query_by_namespace<DB, O, T>(
        namespace: T,
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
                namespace,
                mnemonic,
                fullname,
                cusip,
                fraction,
                quote_flag,
                quote_source,
                quote_tz
                FROM commodities
                WHERE namespace = $1
                "#,
            )
            .bind(namespace),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            .bind(namespace),
            _ => panic!("{kind:?} not support"),
        }
    }
}

#[cfg(feature = "xml")]
use xmltree::Element;
#[cfg(feature = "xml")]

impl Commodity {
    pub(crate) fn new_by_element(e: &Element) -> Self {
        let guid = e
            .get_child("id")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("id must exist");
        let namespace = e
            .get_child("space")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("space must exist");
        let mnemonic = guid.clone();
        let fullname = e
            .get_child("fullname")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let cusip = e
            .get_child("cusip")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let fraction = e
            .get_child("fraction")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| x.parse().expect("must be i32"))
            .unwrap_or(100);
        let quote_flag = match e.get_child("get_quotes") {
            Some(_) => 1,
            None => 0,
        };
        let quote_source = e
            .get_child("quote_source")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let quote_tz = e
            .get_child("quote_tz")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());

        Self {
            guid,
            namespace,
            mnemonic,
            fullname,
            cusip,
            fraction,
            quote_flag,
            quote_source,
            quote_tz,
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
            let result: Vec<Commodity> = rt
                .block_on(async { Commodity::query().fetch_all(&pool).await })
                .unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, rt, kind) = setup();
            let result: Commodity = rt
                .block_on(async {
                    Commodity::query_by_guid("346629655191dcf59a7e2c2a85b70f69", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            assert_eq!("Euro", result.fullname.unwrap());
        }

        #[test]
        fn query_by_namespace() {
            let (pool, rt, kind) = setup();
            let result: Vec<Commodity> = rt
                .block_on(async {
                    Commodity::query_by_namespace("CURRENCY", kind)
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
            let result: Vec<Commodity> = rt
                .block_on(async { Commodity::query().fetch_all(&pool).await })
                .unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, rt, kind) = setup();
            let result: Commodity = rt
                .block_on(async {
                    Commodity::query_by_guid("346629655191dcf59a7e2c2a85b70f69", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            assert_eq!("Euro", result.fullname.unwrap());
        }

        #[test]
        fn query_by_namespace() {
            let (pool, rt, kind) = setup();
            let result: Vec<Commodity> = rt
                .block_on(async {
                    Commodity::query_by_namespace("CURRENCY", kind)
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
            let result: Vec<Commodity> = rt
                .block_on(async { Commodity::query().fetch_all(&pool).await })
                .unwrap();
            assert_eq!(5, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, rt, kind) = setup();
            let result: Commodity = rt
                .block_on(async {
                    Commodity::query_by_guid("346629655191dcf59a7e2c2a85b70f69", kind)
                        .fetch_one(&pool)
                        .await
                })
                .unwrap();
            assert_eq!("Euro", result.fullname.unwrap());
        }

        #[test]
        fn query_by_namespace() {
            let (pool, rt, kind) = setup();
            let result: Vec<Commodity> = rt
                .block_on(async {
                    Commodity::query_by_namespace("CURRENCY", kind)
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
            <gnc:commodity version="2.0.0">
                <cmdty:space>CURRENCY</cmdty:space>
                <cmdty:id>EUR</cmdty:id>
                <cmdty:get_quotes/>
                <cmdty:quote_source>currency</cmdty:quote_source>
                <cmdty:quote_tz/>
            </gnc:commodity>
            </gnc-v2>
            "##;

            let e = Element::parse(data.as_bytes())
                .unwrap()
                .take_child("commodity")
                .unwrap();

            let commodity = Commodity::new_by_element(&e);

            assert_eq!(commodity.guid, "EUR");
            assert_eq!(commodity.namespace, "CURRENCY");
            assert_eq!(commodity.mnemonic, "EUR");
            assert_eq!(commodity.fullname, None);
            assert_eq!(commodity.cusip, None);
            assert_eq!(commodity.fraction, 100);
            assert_eq!(commodity.quote_flag, 1);
            assert_eq!(commodity.quote_source.as_ref().unwrap(), "currency");
            assert_eq!(commodity.quote_tz, None);
        }
    }
}
