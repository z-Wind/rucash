use super::TestSchemas;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::kind::SQLKind;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
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

impl super::NullNone for Account {
    fn null_none(self) -> Self {
        let commodity_guid = self.commodity_guid.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });
        let parent_guid = self.parent_guid.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });
        let code = self.code.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });
        let description = self.description.as_ref().and_then(|x| match x.as_str() {
            "" => None,
            x => Some(x.to_string()),
        });

        let hidden = match self.hidden {
            Some(x) => Some(x),
            None => Some(0),
        };
        let placeholder = match self.placeholder {
            Some(x) => Some(x),
            None => Some(0),
        };

        Self {
            commodity_guid,
            parent_guid,
            code,
            description,
            hidden,
            placeholder,
            ..self
        }
    }
}

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
impl<'q> Account {
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
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }

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
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }

    pub(crate) fn query_by_parent_guid<DB, O, T>(
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
            .bind(guid),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            .bind(guid),
            _ => panic!("{:?} not support", kind),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn query_by_name<DB, O, T>(
        name: T,
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
            .bind(name),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            .bind(name),
            _ => panic!("{:?} not support", kind),
        }
    }

    pub(crate) fn query_like_name<DB, O, T>(
        name: T,
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
            .bind(name),
            SQLKind::MySql | SQLKind::Sqlite => sqlx::query_as(
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
            .bind(name),
            _ => panic!("{:?} not support", kind),
        }
    }
}

#[cfg(feature = "xml")]
use xmltree::Element;
#[cfg(feature = "xml")]
impl Account {
    pub(crate) fn new_by_element(e: &Element) -> Self {
        let guid = e
            .get_child("id")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("id must exist");
        let name = e
            .get_child("name")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("name must exist");
        let account_type = e
            .get_child("type")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("type must exist");
        let commodity_guid = e
            .get_child("commodity")
            .and_then(|x| x.get_child("id"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let commodity_scu = e
            .get_child("commodity-scu")
            .and_then(|x| x.get_text())
            .map(|x| x.parse().expect("must be i32"))
            .unwrap_or(0);
        let non_std_scu = e
            .get_child("non-std-scu")
            .and_then(|x| x.get_text())
            .map(|x| x.parse().expect("must be i32"))
            .unwrap_or(0);
        let parent_guid = e
            .get_child("parent")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let code = e
            .get_child("code")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let description = e
            .get_child("description")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let hidden = e
            .get_child("hidden")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| x.parse().unwrap_or(0));

        let slots: Vec<&Element> = match e.get_child("slots") {
            None => Vec::new(),
            Some(x) => x.children.iter().filter_map(|x| x.as_element()).collect(),
        };
        let placeholder = slots
            .iter()
            .find(|e| {
                e.get_child("key")
                    .and_then(|e| e.get_text())
                    .map(|s| s.into_owned())
                    == Some("placeholder".to_string())
            })
            .and_then(|e| e.get_child("value"))
            .and_then(|s| s.get_text())
            .map(|x| x.into_owned())
            .map(|x| if x == "true" { 1 } else { 0 });

        Self {
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
            placeholder,
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
            let (pool, _) = setup(URI);
            let result: Vec<Account> =
                block_on(async { Account::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(21, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup(URI);
            let result: Account = block_on(async {
                Account::query_by_guid("fcd795021c976ba75621ec39e75f6214", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Asset", result.name);
        }

        #[test]
        fn query_by_commodity_guid() {
            let (pool, kind) = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_commodity_guid("346629655191dcf59a7e2c2a85b70f69", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(14, result.len());
        }

        #[test]
        fn query_by_parent_guid() {
            let (pool, kind) = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_parent_guid("fcd795021c976ba75621ec39e75f6214", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_name() {
            let (pool, kind) = setup(URI);
            let result: Account =
                block_on(async { Account::query_by_name("Asset", kind).fetch_one(&pool).await })
                    .unwrap();
            assert_eq!("fcd795021c976ba75621ec39e75f6214", result.guid);
        }

        #[test]
        fn query_like_name() {
            let (pool, kind) = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_like_name("%AS%", kind)
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
            let result: Vec<Account> =
                block_on(async { Account::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(21, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup(URI);
            let result: Account = block_on(async {
                Account::query_by_guid("fcd795021c976ba75621ec39e75f6214", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Asset", result.name);
        }

        #[test]
        fn query_by_commodity_guid() {
            let (pool, kind) = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_commodity_guid("346629655191dcf59a7e2c2a85b70f69", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(14, result.len());
        }

        #[test]
        fn query_by_parent_guid() {
            let (pool, kind) = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_parent_guid("fcd795021c976ba75621ec39e75f6214", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_name() {
            let (pool, kind) = setup(URI);
            let result: Account =
                block_on(async { Account::query_by_name("Asset", kind).fetch_one(&pool).await })
                    .unwrap();
            assert_eq!("fcd795021c976ba75621ec39e75f6214", result.guid);
        }

        #[test]
        fn query_like_name() {
            let (pool, kind) = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_like_name("%AS%", kind)
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
            let result: Vec<Account> =
                block_on(async { Account::query().fetch_all(&pool).await }).unwrap();
            assert_eq!(21, result.len());
        }

        #[test]
        fn query_by_guid() {
            let (pool, kind) = setup(URI);
            let result: Account = block_on(async {
                Account::query_by_guid("fcd795021c976ba75621ec39e75f6214", kind)
                    .fetch_one(&pool)
                    .await
            })
            .unwrap();
            assert_eq!("Asset", result.name);
        }

        #[test]
        fn query_by_commodity_guid() {
            let (pool, kind) = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_commodity_guid("346629655191dcf59a7e2c2a85b70f69", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(14, result.len());
        }

        #[test]
        fn query_by_parent_guid() {
            let (pool, kind) = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_by_parent_guid("fcd795021c976ba75621ec39e75f6214", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }

        #[test]
        fn query_by_name() {
            let (pool, kind) = setup(URI);
            let result: Account =
                block_on(async { Account::query_by_name("Asset", kind).fetch_one(&pool).await })
                    .unwrap();
            assert_eq!("fcd795021c976ba75621ec39e75f6214", result.guid);
        }

        #[test]
        fn query_like_name() {
            let (pool, kind) = setup(URI);
            let result: Vec<Account> = block_on(async {
                Account::query_like_name("%AS%", kind)
                    .fetch_all(&pool)
                    .await
            })
            .unwrap();
            assert_eq!(3, result.len());
        }
    }

    #[cfg(feature = "xml")]
    mod xml {
        use super::*;
        use std::sync::Arc;

        #[allow(dead_code)]
        const URI: &str = r"tests\db\xml\complex_sample.gnucash";

        #[allow(dead_code)]
        fn setup() -> Arc<Element> {
            crate::XMLBook::new(URI).unwrap().pool.0.clone()
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
            <gnc:account version="2.0.0">
                <act:name>Asset</act:name>
                <act:id type="guid">fcd795021c976ba75621ec39e75f6214</act:id>
                <act:type>ASSET</act:type>
                <act:commodity>
                    <cmdty:space>CURRENCY</cmdty:space>
                    <cmdty:id>EUR</cmdty:id>
                </act:commodity>
                <act:commodity-scu>100</act:commodity-scu>
                <act:slots>
                    <slot>
                    <slot:key>placeholder</slot:key>
                    <slot:value type="string">true</slot:value>
                    </slot>
                </act:slots>
                <act:parent type="guid">00622dda21937b29e494179de5013f82</act:parent>
            </gnc:account>
            </gnc-v2>
            "##;

            let e = Element::parse(data.as_bytes())
                .unwrap()
                .take_child("account")
                .unwrap();

            let account = Account::new_by_element(&e);

            assert_eq!(account.guid, "fcd795021c976ba75621ec39e75f6214");
            assert_eq!(account.name, "Asset");
            assert_eq!(account.account_type, "ASSET");
            assert_eq!(account.commodity_guid.as_ref().unwrap(), "EUR");
            assert_eq!(account.commodity_scu, 100);
            assert_eq!(account.non_std_scu, 0);
            assert_eq!(
                account.parent_guid.as_ref().unwrap(),
                "00622dda21937b29e494179de5013f82"
            );
            assert_eq!(account.code, None);
            assert_eq!(account.description, None);
            assert_eq!(account.hidden, None);
            assert_eq!(account.placeholder.unwrap(), 1);
        }
    }
}
