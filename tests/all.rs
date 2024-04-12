use rucash::{Book, MySQLQuery, PostgreSQLQuery, SQLiteQuery, XMLQuery};

mod mysql;
mod postgresql;
mod sqlite;
mod xml;

mod consistency {
    use super::*;
    use pretty_assertions::assert_eq;

    async fn setup_sqlite() -> Book<SQLiteQuery> {
        let query = SQLiteQuery::new(&sqlite::uri()).await.unwrap();
        Book::new(query).await.unwrap()
    }

    async fn setup_postgresql() -> Book<PostgreSQLQuery> {
        let query = PostgreSQLQuery::new(&postgresql::uri()).await.unwrap();
        Book::new(query).await.unwrap()
    }

    async fn setup_mysql() -> Book<MySQLQuery> {
        let query = MySQLQuery::new(&mysql::uri()).await.unwrap();
        Book::new(query).await.unwrap()
    }

    async fn setup_xml() -> Book<XMLQuery> {
        let query = XMLQuery::new(&xml::uri()).unwrap();
        Book::new(query).await.unwrap()
    }

    fn vec_match<T, Q>(a: &[T], b: &[Q], cmp: fn(&T, &Q) -> bool) -> bool {
        assert_eq!(a.len(), b.len());
        let matching = a.iter().zip(b.iter()).filter(|&(a, b)| cmp(a, b)).count();
        assert_eq!(matching, a.len());
        matching == a.len() && matching == b.len()
    }

    #[tokio::test]
    async fn accounts_consistency() {
        fn cmp<QA: rucash::Query, QB: rucash::Query>(
            a: &rucash::model::Account<QA>,
            b: &rucash::model::Account<QB>,
        ) -> bool {
            // println!("a:{}\nb:{}\n\n", a.guid, b.guid);
            assert_eq!(a.guid, b.guid);
            assert_eq!(a.name, b.name);
            assert_eq!(a.r#type, b.r#type);
            // assert_eq!(a.commodity_guid, b.commodity_guid);
            assert_eq!(a.commodity_scu, b.commodity_scu);
            assert_eq!(a.non_std_scu, b.non_std_scu);
            assert_eq!(a.parent_guid, b.parent_guid);
            assert_eq!(a.code, b.code);
            assert_eq!(a.description, b.description);
            assert_eq!(a.hidden, b.hidden);
            assert_eq!(a.placeholder, b.placeholder);

            true
        }

        let mut v_sqlite = setup_sqlite().await.accounts().await.unwrap();
        v_sqlite.sort_by_key(|x| x.guid.clone());

        let mut v_postgresql = setup_postgresql().await.accounts().await.unwrap();
        v_postgresql.sort_by_key(|x| x.guid.clone());

        let mut v_mysql = setup_mysql().await.accounts().await.unwrap();
        v_mysql.sort_by_key(|x| x.guid.clone());

        let mut v_xml = setup_xml().await.accounts().await.unwrap();
        v_xml.sort_by_key(|x| x.guid.clone());

        println!("vec_match(&v_sqlite, &v_postgresql)");
        assert!(vec_match(&v_sqlite, &v_postgresql, cmp));
        println!("vec_match(&v_sqlite, &v_mysql)");
        assert!(vec_match(&v_sqlite, &v_mysql, cmp));
        println!("vec_match(&v_sqlite, &v_xml)");
        assert!(vec_match(
            &v_sqlite
                .into_iter()
                .filter(|x| x.name != "Template Root")
                .collect::<Vec<_>>(),
            &v_xml,
            cmp
        ));
    }

    #[tokio::test]
    async fn splits_consistency() {
        fn cmp<QA: rucash::Query, QB: rucash::Query>(
            a: &rucash::model::Split<QA>,
            b: &rucash::model::Split<QB>,
        ) -> bool {
            assert_eq!(a.guid, b.guid);
            assert_eq!(a.tx_guid, b.tx_guid);
            assert_eq!(a.account_guid, b.account_guid);
            assert_eq!(a.memo, b.memo);
            assert_eq!(a.action, b.action);
            assert_eq!(a.reconcile_state, b.reconcile_state);
            assert_eq!(a.reconcile_datetime, b.reconcile_datetime);
            assert_eq!(a.value, b.value);
            assert_eq!(a.quantity, b.quantity);
            assert_eq!(a.lot_guid, b.lot_guid);

            true
        }

        let mut v_sqlite = setup_sqlite().await.splits().await.unwrap();
        v_sqlite.sort_by_key(|x| x.guid.clone());

        let mut v_postgresql = setup_postgresql().await.splits().await.unwrap();
        v_postgresql.sort_by_key(|x| x.guid.clone());

        let mut v_mysql = setup_mysql().await.splits().await.unwrap();
        v_mysql.sort_by_key(|x| x.guid.clone());

        let mut v_xml = setup_xml().await.splits().await.unwrap();
        v_xml.sort_by_key(|x| x.guid.clone());

        println!("vec_match(&v_sqlite, &v_postgresql)");
        assert!(vec_match(&v_sqlite, &v_postgresql, cmp));
        println!("vec_match(&v_sqlite, &v_mysql)");
        assert!(vec_match(&v_sqlite, &v_mysql, cmp));
        println!("vec_match(&v_sqlite, &v_xml)");
        assert!(vec_match(&v_sqlite, &v_xml, cmp));
    }

    #[tokio::test]
    async fn transactions_consistency() {
        fn cmp<QA: rucash::Query, QB: rucash::Query>(
            a: &rucash::model::Transaction<QA>,
            b: &rucash::model::Transaction<QB>,
        ) -> bool {
            assert_eq!(a.guid, b.guid);
            // xml guid 會是 USD 之類的
            // assert_eq!(a.currency_guid, b.currency_guid);
            assert_eq!(a.num, b.num);
            assert_eq!(a.post_datetime, b.post_datetime);
            assert_eq!(a.enter_datetime, b.enter_datetime);
            assert_eq!(a.description, b.description);

            true
        }

        let mut v_sqlite = setup_sqlite().await.transactions().await.unwrap();
        v_sqlite.sort_by_key(|x| x.guid.clone());

        let mut v_postgresql = setup_postgresql().await.transactions().await.unwrap();
        v_postgresql.sort_by_key(|x| x.guid.clone());

        let mut v_mysql = setup_mysql().await.transactions().await.unwrap();
        v_mysql.sort_by_key(|x| x.guid.clone());

        let mut v_xml = setup_xml().await.transactions().await.unwrap();
        v_xml.sort_by_key(|x| x.guid.clone());

        println!("vec_match(&v_sqlite, &v_postgresql)");
        assert!(vec_match(&v_sqlite, &v_postgresql, cmp));
        println!("vec_match(&v_sqlite, &v_mysql)");
        assert!(vec_match(&v_sqlite, &v_mysql, cmp));
        println!("vec_match(&v_sqlite, &v_xml)");
        assert!(vec_match(&v_sqlite, &v_xml, cmp));
    }

    #[tokio::test]
    async fn prices_consistency() {
        fn cmp<QA: rucash::Query, QB: rucash::Query>(
            a: &rucash::model::Price<QA>,
            b: &rucash::model::Price<QB>,
        ) -> bool {
            assert_eq!(a.guid, b.guid);
            // xml guid 會是 USD 之類的
            // assert_eq!(a.commodity_guid, b.commodity_guid);
            // xml guid 會是 USD 之類的
            // assert_eq!(a.currency_guid, b.currency_guid);
            assert_eq!(a.datetime, b.datetime);
            assert_eq!(a.source, b.source);
            assert_eq!(a.r#type, b.r#type);
            assert_eq!(a.value, b.value);

            true
        }

        let mut v_sqlite = setup_sqlite().await.prices().await.unwrap();
        v_sqlite.sort_by_key(|x| x.guid.clone());

        let mut v_postgresql = setup_postgresql().await.prices().await.unwrap();
        v_postgresql.sort_by_key(|x| x.guid.clone());

        let mut v_mysql = setup_mysql().await.prices().await.unwrap();
        v_mysql.sort_by_key(|x| x.guid.clone());

        let mut v_xml = setup_xml().await.prices().await.unwrap();
        v_xml.sort_by_key(|x| x.guid.clone());

        println!("vec_match(&v_sqlite, &v_postgresql)");
        assert!(vec_match(&v_sqlite, &v_postgresql, cmp));
        println!("vec_match(&v_sqlite, &v_mysql)");
        assert!(vec_match(&v_sqlite, &v_mysql, cmp));
        println!("vec_match(&v_sqlite, &v_xml)");
        assert!(vec_match(&v_sqlite, &v_xml, cmp));
    }

    #[tokio::test]
    async fn commodities_consistency() {
        fn cmp<QA: rucash::Query, QB: rucash::Query>(
            a: &rucash::model::Commodity<QA>,
            b: &rucash::model::Commodity<QB>,
        ) -> bool {
            println!("a:{}\nb:{}\n\n", a.guid, b.guid);
            // xml guid 會是 USD 之類的
            // assert_eq!(a.guid, b.guid);
            assert_eq!(a.namespace, b.namespace);
            assert_eq!(a.mnemonic, b.mnemonic);
            // xml 無 fullname
            // assert_eq!(a.fullname, b.fullname);
            // xml 無 cusip
            // assert_eq!(a.cusip, b.cusip);
            assert_eq!(a.fraction, b.fraction);
            assert_eq!(a.quote_flag, b.quote_flag);
            // xml 的 price 會沒有 quote_source
            // assert_eq!(a.quote_source, b.quote_source);
            assert_eq!(a.quote_tz, b.quote_tz);

            true
        }

        let mut v_sqlite = setup_sqlite().await.commodities().await.unwrap();
        v_sqlite.sort_by_key(|x| x.mnemonic.clone());

        let mut v_postgresql = setup_postgresql().await.commodities().await.unwrap();
        v_postgresql.sort_by_key(|x| x.mnemonic.clone());

        let mut v_mysql = setup_mysql().await.commodities().await.unwrap();
        v_mysql.sort_by_key(|x| x.mnemonic.clone());

        let mut v_xml = setup_xml().await.commodities().await.unwrap();
        v_xml.sort_by_key(|x| x.mnemonic.clone());

        println!("vec_match(&v_sqlite, &v_postgresql)");
        assert!(vec_match(&v_sqlite, &v_postgresql, cmp));
        println!("vec_match(&v_sqlite, &v_mysql)");
        assert!(vec_match(&v_sqlite, &v_mysql, cmp));
        println!("vec_match(&v_sqlite, &v_xml)");
        assert!(vec_match(
            &v_sqlite,
            &v_xml
                .into_iter()
                .filter(|x| x.guid != "template")
                .collect::<Vec<_>>(),
            cmp
        ),);
    }
}
