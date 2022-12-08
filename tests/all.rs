use rucash::MySQLBook;
use rucash::PostgreSQLBook;
use rucash::SqliteBook;
use rucash::XMLBook;

mod mysql;
mod postgresql;
mod sqlite;
mod xml;

mod consistency {
    use super::*;

    async fn setup_sqlite() -> SqliteBook {
        SqliteBook::new(sqlite::URI).await.unwrap()
    }

    async fn setup_postgresql() -> PostgreSQLBook {
        PostgreSQLBook::new(postgresql::URI).await.unwrap()
    }

    async fn setup_mysql() -> MySQLBook {
        MySQLBook::new(mysql::URI).await.unwrap()
    }

    fn setup_xml() -> XMLBook {
        XMLBook::new(xml::URI).unwrap()
    }

    fn vec_match<T: PartialEq + std::fmt::Debug>(
        a: &Vec<&T>,
        b: &Vec<&T>,
        cmp: fn(&T, &T) -> bool,
    ) -> bool {
        assert_eq!(a.len(), b.len());
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| cmp(a, b)).count();
        assert_eq!(matching, a.len());
        matching == a.len() && matching == b.len()
    }

    #[tokio::test]
    async fn accounts_consistency() {
        fn cmp(a: &rucash::model::Account, b: &rucash::model::Account) -> bool {
            // println!("a:{}\nb:{}\n\n", a.guid, b.guid);
            assert_eq!(a.guid, b.guid);
            assert_eq!(a.name, b.name);
            assert_eq!(a.account_type, b.account_type);
            if a.commodity_guid.as_ref().or(b.commodity_guid.as_ref()) == None {
                assert_eq!(a.commodity_guid, b.commodity_guid);
            }
            assert_eq!(a.commodity_scu, b.commodity_scu);
            assert_eq!(a.non_std_scu, b.non_std_scu);
            assert_eq!(a.parent_guid, b.parent_guid);
            assert_eq!(a.code, b.code);
            assert_eq!(a.description, b.description);
            assert_eq!(a.hidden, b.hidden);
            assert_eq!(a.placeholder, b.placeholder);

            true
        }

        let v_sqlite = setup_sqlite().await.accounts().await.unwrap();
        let mut v_sqlite: Vec<&rucash::model::Account> =
            v_sqlite.iter().map(|v| v.content()).collect();
        v_sqlite.sort_by_key(|x| &x.guid);

        let v_postgresql = setup_postgresql().await.accounts().await.unwrap();
        let mut v_postgresql: Vec<&rucash::model::Account> =
            v_postgresql.iter().map(|v| v.content()).collect();
        v_postgresql.sort_by_key(|x| &x.guid);

        let v_mysql = setup_mysql().await.accounts().await.unwrap();
        let mut v_mysql: Vec<&rucash::model::Account> =
            v_mysql.iter().map(|v| v.content()).collect();
        v_mysql.sort_by_key(|x| &x.guid);

        let v_xml = setup_xml().accounts();
        let mut v_xml: Vec<&rucash::model::Account> = v_xml.iter().map(|v| v.content()).collect();
        v_xml.sort_by_key(|x| &x.guid);

        println!("vec_match(&v_sqlite, &v_postgresql)");
        assert_eq!(vec_match(&v_sqlite, &v_postgresql, cmp), true);
        println!("vec_match(&v_sqlite, &v_mysql)");
        assert_eq!(vec_match(&v_sqlite, &v_mysql, cmp), true);
        println!("vec_match(&v_sqlite, &v_xml)");
        assert_eq!(
            vec_match(
                &v_sqlite
                    .into_iter()
                    .filter(|x| x.name != "Template Root")
                    .collect(),
                &v_xml,
                cmp
            ),
            true
        );
    }

    #[tokio::test]
    async fn splits_consistency() {
        fn cmp(a: &rucash::model::Split, b: &rucash::model::Split) -> bool {
            assert_eq!(a, b);
            a == b
        }

        let v_sqlite = setup_sqlite().await.splits().await.unwrap();
        let mut v_sqlite: Vec<&rucash::model::Split> =
            v_sqlite.iter().map(|v| v.content()).collect();
        v_sqlite.sort_by_key(|x| &x.guid);

        let v_postgresql = setup_postgresql().await.splits().await.unwrap();
        let mut v_postgresql: Vec<&rucash::model::Split> =
            v_postgresql.iter().map(|v| v.content()).collect();
        v_postgresql.sort_by_key(|x| &x.guid);

        let v_mysql = setup_mysql().await.splits().await.unwrap();
        let mut v_mysql: Vec<&rucash::model::Split> = v_mysql.iter().map(|v| v.content()).collect();
        v_mysql.sort_by_key(|x| &x.guid);

        let v_xml = setup_xml().splits();
        let mut v_xml: Vec<&rucash::model::Split> = v_xml.iter().map(|v| v.content()).collect();
        v_xml.sort_by_key(|x| &x.guid);

        println!("vec_match(&v_sqlite, &v_postgresql)");
        assert_eq!(vec_match(&v_sqlite, &v_postgresql, cmp), true);
        println!("vec_match(&v_sqlite, &v_mysql)");
        assert_eq!(vec_match(&v_sqlite, &v_mysql, cmp), true);
        println!("vec_match(&v_sqlite, &v_xml)");
        assert_eq!(vec_match(&v_sqlite, &v_xml, cmp), true);
    }

    #[tokio::test]
    async fn transactions_consistency() {
        fn cmp(a: &rucash::model::Transaction, b: &rucash::model::Transaction) -> bool {
            assert_eq!(a.guid, b.guid);
            // assert_eq!(a.currency_guid, b.currency_guid);
            assert_eq!(a.num, b.num);
            assert_eq!(a.post_date, b.post_date);
            assert_eq!(a.enter_date, b.enter_date);
            assert_eq!(a.description, b.description);

            true
        }

        let v_sqlite = setup_sqlite().await.transactions().await.unwrap();
        let mut v_sqlite: Vec<&rucash::model::Transaction> =
            v_sqlite.iter().map(|v| v.content()).collect();
        v_sqlite.sort_by_key(|x| &x.guid);

        let v_postgresql = setup_postgresql().await.transactions().await.unwrap();
        let mut v_postgresql: Vec<&rucash::model::Transaction> =
            v_postgresql.iter().map(|v| v.content()).collect();
        v_postgresql.sort_by_key(|x| &x.guid);

        let v_mysql = setup_mysql().await.transactions().await.unwrap();
        let mut v_mysql: Vec<&rucash::model::Transaction> =
            v_mysql.iter().map(|v| v.content()).collect();
        v_mysql.sort_by_key(|x| &x.guid);

        let v_xml = setup_xml().transactions();
        let mut v_xml: Vec<&rucash::model::Transaction> =
            v_xml.iter().map(|v| v.content()).collect();
        v_xml.sort_by_key(|x| &x.guid);

        println!("vec_match(&v_sqlite, &v_postgresql)");
        assert_eq!(vec_match(&v_sqlite, &v_postgresql, cmp), true);
        println!("vec_match(&v_sqlite, &v_mysql)");
        assert_eq!(vec_match(&v_sqlite, &v_mysql, cmp), true);
        println!("vec_match(&v_sqlite, &v_xml)");
        assert_eq!(vec_match(&v_sqlite, &v_xml, cmp), true);
    }

    #[tokio::test]
    async fn prices_consistency() {
        fn cmp(a: &rucash::model::Price, b: &rucash::model::Price) -> bool {
            assert_eq!(a.guid, b.guid);
            // assert_eq!(a.commodity_guid, b.commodity_guid);
            // assert_eq!(a.currency_guid, b.currency_guid);
            assert_eq!(a.date, b.date);
            assert_eq!(a.source, b.source);
            assert_eq!(a.r#type, b.r#type);
            assert_eq!(a.value_num, b.value_num);
            assert_eq!(a.value_denom, b.value_denom);
            assert_eq!(a.value(), b.value());

            true
        }

        let v_sqlite = setup_sqlite().await.prices().await.unwrap();
        let mut v_sqlite: Vec<&rucash::model::Price> =
            v_sqlite.iter().map(|v| v.content()).collect();
        v_sqlite.sort_by_key(|x| &x.guid);

        let v_postgresql = setup_postgresql().await.prices().await.unwrap();
        let mut v_postgresql: Vec<&rucash::model::Price> =
            v_postgresql.iter().map(|v| v.content()).collect();
        v_postgresql.sort_by_key(|x| &x.guid);

        let v_mysql = setup_mysql().await.prices().await.unwrap();
        let mut v_mysql: Vec<&rucash::model::Price> = v_mysql.iter().map(|v| v.content()).collect();
        v_mysql.sort_by_key(|x| &x.guid);

        let v_xml = setup_xml().prices();
        let mut v_xml: Vec<&rucash::model::Price> = v_xml.iter().map(|v| v.content()).collect();
        v_xml.sort_by_key(|x| &x.guid);

        println!("vec_match(&v_sqlite, &v_postgresql)");
        assert_eq!(vec_match(&v_sqlite, &v_postgresql, cmp), true);
        println!("vec_match(&v_sqlite, &v_mysql)");
        assert_eq!(vec_match(&v_sqlite, &v_mysql, cmp), true);
        println!("vec_match(&v_sqlite, &v_xml)");
        assert_eq!(vec_match(&v_sqlite, &v_xml, cmp), true);
    }

    #[tokio::test]
    async fn commodities_consistency() {
        fn cmp(a: &rucash::model::Commodity, b: &rucash::model::Commodity) -> bool {
            println!("a:{}\nb:{}\n\n", a.guid, b.guid);
            // assert_eq!(a.guid, b.guid);
            assert_eq!(a.namespace, b.namespace);
            assert_eq!(a.mnemonic, b.mnemonic);
            // assert_eq!(a.fullname, b.fullname);
            // assert_eq!(a.cusip, b.cusip);
            assert_eq!(a.fraction, b.fraction);
            assert_eq!(a.quote_flag, b.quote_flag);
            // assert_eq!(a.quote_source, b.quote_source);
            assert_eq!(a.quote_tz, b.quote_tz);

            true
        }

        let v_sqlite = setup_sqlite().await.commodities().await.unwrap();
        let mut v_sqlite: Vec<&rucash::model::Commodity> =
            v_sqlite.iter().map(|v| v.content()).collect();
        v_sqlite.sort_by_key(|x| &x.mnemonic);

        let v_postgresql = setup_postgresql().await.commodities().await.unwrap();
        let mut v_postgresql: Vec<&rucash::model::Commodity> =
            v_postgresql.iter().map(|v| v.content()).collect();
        v_postgresql.sort_by_key(|x| &x.mnemonic);

        let v_mysql = setup_mysql().await.commodities().await.unwrap();
        let mut v_mysql: Vec<&rucash::model::Commodity> =
            v_mysql.iter().map(|v| v.content()).collect();
        v_mysql.sort_by_key(|x| &x.mnemonic);

        let v_xml = setup_xml().commodities();
        let mut v_xml: Vec<&rucash::model::Commodity> = v_xml.iter().map(|v| v.content()).collect();
        v_xml.sort_by_key(|x| &x.mnemonic);

        println!("vec_match(&v_sqlite, &v_postgresql)");
        assert_eq!(vec_match(&v_sqlite, &v_postgresql, cmp), true);
        println!("vec_match(&v_sqlite, &v_mysql)");
        assert_eq!(vec_match(&v_sqlite, &v_mysql, cmp), true);
        println!("vec_match(&v_sqlite, &v_xml)");
        assert_eq!(
            vec_match(
                &v_sqlite,
                &v_xml.into_iter().filter(|x| x.guid != "template").collect(),
                cmp
            ),
            true
        );
    }
}
