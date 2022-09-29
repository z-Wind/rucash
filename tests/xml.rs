use rucash::XMLBook;

pub const URI: &str = "tests/db/xml/complex_sample.gnucash";

mod book {
    use super::*;

    #[test]
    fn new() {
        XMLBook::new(URI).unwrap();
    }

    #[test]
    #[should_panic]
    fn new_fail() {
        XMLBook::new("./tests/db/xml/aa").unwrap();
    }

    #[test]
    fn accounts() {
        let book = XMLBook::new(URI).unwrap();
        let accounts = book.accounts();
        // 21 - 1 template
        assert_eq!(accounts.len(), 20);
    }

    #[test]
    fn accounts_filter() {
        let book = XMLBook::new(URI).unwrap();
        let accounts: Vec<_> = book
            .accounts()
            .into_iter()
            .filter(|x| x.name.to_lowercase().contains(&"aS".to_lowercase()))
            .collect();
        assert_eq!(accounts.len(), 3);
    }

    #[test]
    fn accounts_by_name() {
        let book = XMLBook::new(URI).unwrap();
        let accounts = book.accounts_contains_name("aS");
        assert_eq!(accounts.len(), 3);
    }

    #[test]
    fn account_by_name() {
        let book = XMLBook::new(URI).unwrap();
        let account = book.account_by_name("aS").expect("must have one");
        assert_eq!(account.name, "NASDAQ");
    }

    #[test]
    fn splits() {
        let book = XMLBook::new(URI).unwrap();
        let splits = book.splits();
        assert_eq!(splits.len(), 25);
    }

    #[test]
    fn transactions() {
        let book = XMLBook::new(URI).unwrap();
        let transactions = book.transactions();
        assert_eq!(transactions.len(), 11);
    }

    #[test]
    fn prices() {
        let book = XMLBook::new(URI).unwrap();
        let prices = book.prices();
        assert_eq!(prices.len(), 5);
    }

    #[test]
    fn commodities() {
        let book = XMLBook::new(URI).unwrap();
        let commodities = book.commodities();
        //  6 = 5 + 1 template
        assert_eq!(commodities.len(), 6);
    }

    #[test]
    fn currencies() {
        let book = XMLBook::new(URI).unwrap();
        let currencies = book.currencies();
        assert_eq!(currencies.len(), 4);
    }
}
mod account {
    use super::*;
    #[test]
    fn property() {
        let book = XMLBook::new(URI).unwrap();
        let account = book
            .accounts()
            .into_iter()
            .filter(|x| x.guid == "fcd795021c976ba75621ec39e75f6214")
            .next()
            .unwrap();

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
        assert_eq!(account.hidden, Some(0));
        assert_eq!(account.placeholder.unwrap(), 1);
    }

    #[test]
    fn balance() {
        let book = XMLBook::new(URI).unwrap();
        let account = book
            .accounts()
            .into_iter()
            .filter(|x| x.name == "Current")
            .next()
            .unwrap();

        assert_eq!(account.balance(), 4590.0);
    }
    #[test]
    fn balance_diff_currency() {
        let book = XMLBook::new(URI).unwrap();
        let account = book
            .accounts()
            .into_iter()
            .filter(|x| x.name == "Asset")
            .next()
            .unwrap();

        assert_eq!(account.balance(), 24695.3);
    }
    #[test]
    fn splits() {
        let book = XMLBook::new(URI).unwrap();
        let account = book.account_by_name("Cash").expect("must have one");
        let splits = account.splits();
        assert_eq!(splits.len(), 3);
    }

    #[test]
    fn parent() {
        let book = XMLBook::new(URI).unwrap();
        let account = book.account_by_name("Cash").expect("must have one");
        let parent = account.parent().unwrap();
        assert_eq!(parent.name, "Current");
    }

    #[test]
    fn no_parent() {
        let book = XMLBook::new(URI).unwrap();
        let account = book.account_by_name("Root Account").expect("must have one");
        let parent = account.parent();
        assert!(parent.is_none());
    }

    #[test]
    fn children() {
        let book = XMLBook::new(URI).unwrap();
        let account = book.account_by_name("Current").expect("must have one");
        let children = account.children();
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn commodity() {
        let book = XMLBook::new(URI).unwrap();
        let account = book.account_by_name("Cash").expect("must have one");
        let commodity = account.commodity().unwrap();
        assert_eq!(commodity.mnemonic, "EUR");
    }
}

mod split {
    use super::*;
    #[test]
    fn property() {
        let book = XMLBook::new(URI).unwrap();
        let split = book
            .splits()
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
        assert_eq!(split.value(), 150.0);
        assert_eq!(split.quantity_num, 15000);
        assert_eq!(split.quantity_denom, 100);
        assert_eq!(split.quantity(), 150.0);
        assert_eq!(split.lot_guid, None);
    }
    #[test]
    fn transaction() {
        let book = XMLBook::new(URI).unwrap();
        let split = book
            .splits()
            .into_iter()
            .filter(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
            .next()
            .unwrap();
        let transaction = split.transaction();
        assert_eq!(transaction.description.as_ref().unwrap(), "income 1");
    }

    #[test]
    fn account() {
        let book = XMLBook::new(URI).unwrap();
        let split = book
            .splits()
            .into_iter()
            .filter(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
            .next()
            .unwrap();
        let account = split.account();
        assert_eq!(account.name, "Cash");
    }
}

mod transaction {
    use super::*;
    #[test]
    fn property() {
        let book = XMLBook::new(URI).unwrap();
        let transaction = book
            .transactions()
            .into_iter()
            .filter(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
            .next()
            .unwrap();

        assert_eq!(transaction.guid, "6c8876003c4a6026e38e3afb67d6f2b1");
        assert_eq!(transaction.currency_guid, "EUR");
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
        let book = XMLBook::new(URI).unwrap();
        let transaction = book
            .transactions()
            .into_iter()
            .filter(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
            .next()
            .unwrap();
        let currency = transaction.currency();
        assert_eq!(currency.mnemonic, "EUR");
    }

    #[test]
    fn splits() {
        let book = XMLBook::new(URI).unwrap();
        let transaction = book
            .transactions()
            .into_iter()
            .filter(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
            .next()
            .unwrap();
        let splits = transaction.splits();
        assert_eq!(splits.len(), 2);
    }
}

mod price {
    use super::*;
    #[test]
    fn property() {
        let book = XMLBook::new(URI).unwrap();
        let price = book
            .prices()
            .into_iter()
            .filter(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
            .next()
            .unwrap();

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
        assert_eq!(price.value(), 1.5);
    }

    #[test]
    fn commodity() {
        let book = XMLBook::new(URI).unwrap();
        let price = book
            .prices()
            .into_iter()
            .filter(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
            .next()
            .unwrap();
        let commodity = price.commodity();
        assert_eq!(commodity.mnemonic, "ADF");
    }

    #[test]
    fn currency() {
        let book = XMLBook::new(URI).unwrap();
        let price = book
            .prices()
            .into_iter()
            .filter(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
            .next()
            .unwrap();
        let currency = price.currency();
        assert_eq!(currency.mnemonic, "AED");
    }
}

mod commodity {
    use super::*;
    use float_cmp::assert_approx_eq;

    #[test]
    fn property() {
        let book = XMLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "EUR")
            .next()
            .unwrap();

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

    #[test]
    fn accounts() {
        let book = XMLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "EUR")
            .next()
            .unwrap();
        let accounts = commodity.accounts();
        assert_eq!(accounts.len(), 14);
    }

    #[test]
    fn transactions() {
        let book = XMLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "EUR")
            .next()
            .unwrap();
        let transactions = commodity.transactions();
        assert_eq!(transactions.len(), 11);
    }

    #[test]
    fn as_commodity_prices() {
        let book = XMLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "EUR")
            .next()
            .unwrap();
        let prices = commodity.as_commodity_prices();
        assert_eq!(prices.len(), 1);
    }

    #[test]
    fn as_currency_prices() {
        let book = XMLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "EUR")
            .next()
            .unwrap();
        let prices = commodity.as_currency_prices();
        assert_eq!(prices.len(), 2);
    }

    #[test]
    fn as_commodity_or_currency_prices() {
        let book = XMLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "EUR")
            .next()
            .unwrap();
        let prices = commodity.as_commodity_or_currency_prices();
        assert_eq!(prices.len(), 3);
    }

    #[test]
    fn rate_direct() {
        // ADF => AED
        let book = XMLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "ADF")
            .next()
            .unwrap();
        let currency = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "AED")
            .next()
            .unwrap();

        let rate = commodity.sell(&currency).unwrap();
        assert_eq!(rate, 1.5);
        let rate = currency.buy(&commodity).unwrap();
        assert_eq!(rate, 1.5);

        // AED => EUR
        let book = XMLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "AED")
            .next()
            .unwrap();
        let currency = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "EUR")
            .next()
            .unwrap();

        let rate = commodity.sell(&currency).unwrap();
        assert_approx_eq!(f64, rate, 9.0 / 10.0);
        let rate = currency.buy(&commodity).unwrap();
        assert_approx_eq!(f64, rate, 9.0 / 10.0);
    }

    #[test]
    fn rate_indirect() {
        let book = XMLBook::new(URI).unwrap();
        let commodity = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "USD")
            .next()
            .unwrap();
        let currency = book
            .commodities()
            .into_iter()
            .filter(|x| x.guid == "AED")
            .next()
            .unwrap();

        let rate = commodity.sell(&currency).unwrap();
        assert_approx_eq!(f64, rate, 7.0 / 5.0 * 10.0 / 9.0);
    }
}
