use rucash::MySQLBook;

pub const URI: &str = "mysql://user:secret@localhost/complex_sample.gnucash";

mod book {
    use super::*;

    #[test]
    fn new() {
        MySQLBook::new(URI).unwrap();
    }

    #[test]
    #[should_panic]
    fn new_fail() {
        MySQLBook::new("mysql://user@localhost").unwrap();
    }

    #[test]
    fn accounts() {
        let book = MySQLBook::new(URI).unwrap();
        let accounts = book.accounts().unwrap();
        assert_eq!(accounts.len(), 21);
    }

    #[test]
    fn accounts_filter() {
        let book = MySQLBook::new(URI).unwrap();
        let accounts: Vec<_> = book
            .accounts()
            .unwrap()
            .into_iter()
            .filter(|x| x.name.to_lowercase().contains(&"aS".to_lowercase()))
            .collect();
        assert_eq!(accounts.len(), 3);
    }

    #[test]
    fn accounts_by_name() {
        let book = MySQLBook::new(URI).unwrap();
        let accounts = book.accounts_contains_name("aS").unwrap();
        assert_eq!(accounts.len(), 3);
    }

    #[test]
    fn account_by_name() {
        let book = MySQLBook::new(URI).unwrap();
        let account = book.account_by_name("aS").unwrap().unwrap();
        assert_eq!(account.name, "Asset");
    }

    #[test]
    fn splits() {
        let book = MySQLBook::new(URI).unwrap();
        let splits = book.splits().unwrap();
        assert_eq!(splits.len(), 25);
    }

    #[test]
    fn transactions() {
        let book = MySQLBook::new(URI).unwrap();
        let transactions = book.transactions().unwrap();
        assert_eq!(transactions.len(), 11);
    }

    #[test]
    fn prices() {
        let book = MySQLBook::new(URI).unwrap();
        let prices = book.prices().unwrap();
        assert_eq!(prices.len(), 5);
    }

    #[test]
    fn commodities() {
        let book = MySQLBook::new(URI).unwrap();
        let commodities = book.commodities().unwrap();
        assert_eq!(commodities.len(), 5);
    }

    #[test]
    fn currencies() {
        let book = MySQLBook::new(URI).unwrap();
        let currencies = book.currencies().unwrap();
        assert_eq!(currencies.len(), 4);
    }
}
mod account {
    use super::*;
    #[test]
    fn property() {
        let book = MySQLBook::new(URI).unwrap();
        let account = book
            .accounts()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "fcd795021c976ba75621ec39e75f6214")
            .next()
            .unwrap();

        assert_eq!(account.guid, "fcd795021c976ba75621ec39e75f6214");
        assert_eq!(account.name, "Asset");
        assert_eq!(account.account_type, "ASSET");
        assert_eq!(
            account.commodity_guid.as_ref().unwrap(),
            "346629655191dcf59a7e2c2a85b70f69"
        );
        assert_eq!(account.commodity_scu, 100);
        assert_eq!(account.non_std_scu, 0);
        assert_eq!(
            account.parent_guid.as_ref().unwrap(),
            "00622dda21937b29e494179de5013f82"
        );
        assert_eq!(account.code, None);
        assert_eq!(account.description, None);
        assert_eq!(account.hidden.unwrap(), 0);
        assert_eq!(account.placeholder.unwrap(), 1);
    }

    #[test]
    fn balance() {
        let book = MySQLBook::new(URI).unwrap();
        let account = book
            .accounts()
            .unwrap()
            .into_iter()
            .filter(|x| x.name == "Current")
            .next()
            .unwrap();

        assert_eq!(account.balance().unwrap(), 4590.0);
    }
    #[test]
    fn balance_diff_currency() {
        let book = MySQLBook::new(URI).unwrap();
        let account = book
            .accounts()
            .unwrap()
            .into_iter()
            .filter(|x| x.name == "Asset")
            .next()
            .unwrap();

        assert_eq!(account.balance().unwrap(), 24695.3);
    }
    #[test]
    fn splits() {
        let book = MySQLBook::new(URI).unwrap();
        let account = book.account_by_name("Cash").unwrap().unwrap();
        let splits = account.splits().unwrap();
        assert_eq!(splits.len(), 3);
    }

    #[test]
    fn parent() {
        let book = MySQLBook::new(URI).unwrap();
        let account = book.account_by_name("Cash").unwrap().unwrap();
        let parent = account.parent().unwrap();
        assert_eq!(parent.name, "Current");
    }

    #[test]
    fn no_parent() {
        let book = MySQLBook::new(URI).unwrap();
        let account = book.account_by_name("Root Account").unwrap().unwrap();
        let parent = account.parent();
        assert!(parent.is_none());
    }

    #[test]
    fn children() {
        let book = MySQLBook::new(URI).unwrap();
        let account = book.account_by_name("Current").unwrap().unwrap();
        let children = account.children().unwrap();
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn commodity() {
        let book = MySQLBook::new(URI).unwrap();
        let account = book.account_by_name("Cash").unwrap().unwrap();
        let commodity = account.commodity().unwrap();
        assert_eq!(commodity.mnemonic, "EUR");
    }
}

mod split {
    use super::*;
    #[test]
    fn property() {
        let book = MySQLBook::new(URI).unwrap();
        let split = book
            .splits()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
            .next()
            .unwrap();

        assert_eq!(split.guid, "de832fe97e37811a7fff7e28b3a43425");
        assert_eq!(split.tx_guid, "6c8876003c4a6026e38e3afb67d6f2b1");
        assert_eq!(split.account_guid, "93fc043c3062aaa1297b30e543d2cd0d");
        assert_eq!(split.memo, "");
        assert_eq!(split.action, "");
        assert_eq!(split.reconcile_state, "n");
        assert_eq!(split.reconcile_date, None);
        assert_eq!(split.value_num, 15000);
        assert_eq!(split.value_denom, 100);
        assert_eq!(split.value, 150.0);
        assert_eq!(split.quantity_num, 15000);
        assert_eq!(split.quantity_denom, 100);
        assert_eq!(split.quantity, 150.0);
        assert_eq!(split.lot_guid, None);
    }
    #[test]
    fn transaction() {
        let book = MySQLBook::new(URI).unwrap();
        let split = book
            .splits()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
            .next()
            .unwrap();
        let transaction = split.transaction().unwrap();
        assert_eq!(transaction.description.as_ref().unwrap(), "income 1");
    }

    #[test]
    fn account() {
        let book = MySQLBook::new(URI).unwrap();
        let split = book
            .splits()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
            .next()
            .unwrap();
        let account = split.account().unwrap();
        assert_eq!(account.name, "Cash");
    }
}

mod transaction {
    use super::*;
    #[test]
    fn property() {
        let book = MySQLBook::new(URI).unwrap();
        let transaction = book
            .transactions()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
            .next()
            .unwrap();

        assert_eq!(transaction.guid, "6c8876003c4a6026e38e3afb67d6f2b1");
        assert_eq!(
            transaction.currency_guid,
            "346629655191dcf59a7e2c2a85b70f69"
        );
        assert_eq!(transaction.num, "");
        assert_eq!(
            transaction
                .post_date
                .as_ref()
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            "2014-12-24 10:59:00"
        );
        assert_eq!(
            transaction
                .enter_date
                .as_ref()
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            "2014-12-25 10:08:15"
        );
        assert_eq!(transaction.description.as_ref().unwrap(), "income 1");
    }

    #[test]
    fn currency() {
        let book = MySQLBook::new(URI).unwrap();
        let transaction = book
            .transactions()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
            .next()
            .unwrap();
        let currency = transaction.currency().unwrap();
        assert_eq!(currency.fullname.as_ref().unwrap(), "Euro");
    }

    #[test]
    fn splits() {
        let book = MySQLBook::new(URI).unwrap();
        let transaction = book
            .transactions()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
            .next()
            .unwrap();
        let splits = transaction.splits().unwrap();
        assert_eq!(splits.len(), 2);
    }
}

mod price {
    use super::*;
    #[test]
    fn property() {
        let book = MySQLBook::new(URI).unwrap();
        let price = book
            .prices()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
            .next()
            .unwrap();

        assert_eq!(price.guid, "0d6684f44fb018e882de76094ed9c433");
        assert_eq!(price.commodity_guid, "d821d6776fde9f7c2d01b67876406fd3");
        assert_eq!(price.currency_guid, "5f586908098232e67edb1371408bfaa8");
        assert_eq!(
            price.date.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2018-02-20 23:00:00"
        );
        assert_eq!(price.source.as_ref().unwrap(), "user:price-editor");
        assert_eq!(price.r#type.as_ref().unwrap(), "unknown");
        assert_eq!(price.value_num, 3);
        assert_eq!(price.value_denom, 2);
        assert_eq!(price.value, 1.5);
    }

    #[test]
    fn commodity() {
        let book = MySQLBook::new(URI).unwrap();
        let price = book
            .prices()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
            .next()
            .unwrap();
        let commodity = price.commodity().unwrap();
        assert_eq!(commodity.fullname.as_ref().unwrap(), "Andorran Franc");
    }

    #[test]
    fn currency() {
        let book = MySQLBook::new(URI).unwrap();
        let price = book
            .prices()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
            .next()
            .unwrap();
        let currency = price.currency().unwrap();
        assert_eq!(currency.fullname.as_ref().unwrap(), "UAE Dirham");
    }
}

mod commodity {
    use super::*;
    use float_cmp::assert_approx_eq;

    #[test]
    fn property() {
        let book = MySQLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .next()
            .unwrap();

        assert_eq!(commodity.guid, "346629655191dcf59a7e2c2a85b70f69");
        assert_eq!(commodity.namespace, "CURRENCY");
        assert_eq!(commodity.mnemonic, "EUR");
        assert_eq!(commodity.fullname.as_ref().unwrap(), "Euro");
        assert_eq!(commodity.cusip.as_ref().unwrap(), "978");
        assert_eq!(commodity.fraction, 100);
        assert_eq!(commodity.quote_flag, 1);
        assert_eq!(commodity.quote_source.as_ref().unwrap(), "currency");
        assert_eq!(commodity.quote_tz, None);
    }

    #[test]
    fn accounts() {
        let book = MySQLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .next()
            .unwrap();
        let accounts = commodity.accounts().unwrap();
        assert_eq!(accounts.len(), 14);
    }

    #[test]
    fn transactions() {
        let book = MySQLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .next()
            .unwrap();
        let transactions = commodity.transactions().unwrap();
        assert_eq!(transactions.len(), 11);
    }

    #[test]
    fn as_commodity_prices() {
        let book = MySQLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .next()
            .unwrap();
        let prices = commodity.as_commodity_prices().unwrap();
        assert_eq!(prices.len(), 1);
    }

    #[test]
    fn as_currency_prices() {
        let book = MySQLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .next()
            .unwrap();
        let prices = commodity.as_currency_prices().unwrap();
        assert_eq!(prices.len(), 2);
    }

    #[test]
    fn as_commodity_or_currency_prices() {
        let book = MySQLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .next()
            .unwrap();
        let prices = commodity.as_commodity_or_currency_prices().unwrap();
        assert_eq!(prices.len(), 3);
    }

    #[test]
    fn rate_direct() {
        // ADF => AED
        let book = MySQLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "d821d6776fde9f7c2d01b67876406fd3")
            .next()
            .unwrap();
        let currency = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
            .next()
            .unwrap();

        let rate = commodity.sell(&currency).unwrap();
        assert_eq!(rate, 1.5);
        let rate = currency.buy(&commodity).unwrap();
        assert_eq!(rate, 1.5);

        // AED => EUR
        let book = MySQLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
            .next()
            .unwrap();
        let currency = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .next()
            .unwrap();

        let rate = commodity.sell(&currency).unwrap();
        assert_approx_eq!(f64, rate, 9.0 / 10.0);
        let rate = currency.buy(&commodity).unwrap();
        assert_approx_eq!(f64, rate, 9.0 / 10.0);
    }

    #[test]
    fn rate_indirect() {
        let book = MySQLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "1e5d65e2726a5d4595741cb204992991")
            .next()
            .unwrap();
        let currency = book
            .commodities()
            .unwrap()
            .into_iter()
            .filter(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
            .next()
            .unwrap();

        let rate = commodity.sell(&currency).unwrap();
        assert_approx_eq!(f64, rate, 7.0 / 5.0 * 10.0 / 9.0);
    }
}
