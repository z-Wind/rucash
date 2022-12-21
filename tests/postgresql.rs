use rucash::PostgreSQLBook;
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;

pub const URI: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";

mod book {
    use super::*;

    #[tokio::test]
    async fn new() {
        PostgreSQLBook::new(URI).await.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn new_fail() {
        PostgreSQLBook::new("postgresql://complex_sample.gnucash")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn accounts() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let accounts = book.accounts().await.unwrap();
        assert_eq!(accounts.len(), 21);
    }

    #[tokio::test]
    async fn accounts_filter() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let accounts = book
            .accounts()
            .await
            .unwrap()
            .into_iter()
            .filter(|x| x.name.to_lowercase().contains(&"aS".to_lowercase()));
        assert_eq!(accounts.count(), 3);
    }

    #[tokio::test]
    async fn accounts_by_name() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let accounts = book.accounts_contains_name("aS").await.unwrap();
        assert_eq!(accounts.len(), 3);
    }

    #[tokio::test]
    async fn account_by_name() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let account = book.account_by_name("aS").await.unwrap().unwrap();
        assert_eq!(account.name, "NASDAQ");
    }

    #[tokio::test]
    async fn splits() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let splits = book.splits().await.unwrap();
        assert_eq!(splits.len(), 25);
    }

    #[tokio::test]
    async fn transactions() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let transactions = book.transactions().await.unwrap();
        assert_eq!(transactions.len(), 11);
    }

    #[tokio::test]
    async fn prices() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let prices = book.prices().await.unwrap();
        assert_eq!(prices.len(), 5);
    }

    #[tokio::test]
    async fn commodities() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let commodities = book.commodities().await.unwrap();
        assert_eq!(commodities.len(), 5);
    }

    #[tokio::test]
    async fn currencies() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let currencies = book.currencies().await.unwrap();
        assert_eq!(currencies.len(), 4);
    }
}
mod account {
    use super::*;
    #[tokio::test]
    async fn property() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let account = book
            .accounts()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "fcd795021c976ba75621ec39e75f6214")
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

    #[tokio::test]
    async fn balance() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let account = book
            .accounts()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.name == "Current")
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_eq!(account.balance().await.unwrap(), 4590.0);
        #[cfg(feature = "decimal")]
        assert_eq!(account.balance().await.unwrap(), Decimal::new(4590, 0));
    }
    #[tokio::test]
    async fn balance_diff_currency() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let account = book
            .accounts()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.name == "Asset")
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_eq!(account.balance().await.unwrap(), 24695.3);
        #[cfg(feature = "decimal")]
        assert_eq!(account.balance().await.unwrap(), Decimal::new(246953, 1));
    }
    #[tokio::test]
    async fn splits() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let account = book.account_by_name("Cash").await.unwrap().unwrap();
        let splits = account.splits().await.unwrap();
        assert_eq!(splits.len(), 3);
    }

    #[tokio::test]
    async fn parent() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let account = book.account_by_name("Cash").await.unwrap().unwrap();
        let parent = account.parent().await.unwrap();
        assert_eq!(parent.name, "Current");
    }

    #[tokio::test]
    async fn no_parent() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let account = book.account_by_name("Root Account").await.unwrap().unwrap();
        let parent = account.parent().await;
        assert!(parent.is_none());
    }

    #[tokio::test]
    async fn children() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let account = book.account_by_name("Current").await.unwrap().unwrap();
        let children = account.children().await.unwrap();
        assert_eq!(children.len(), 3);
    }

    #[tokio::test]
    async fn commodity() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let account = book.account_by_name("Cash").await.unwrap().unwrap();
        let commodity = account.commodity().await.unwrap();
        assert_eq!(commodity.mnemonic, "EUR");
    }
}

mod split {
    use super::*;
    #[tokio::test]
    async fn property() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let split = book
            .splits()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
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

        #[cfg(not(feature = "decimal"))]
        assert_eq!(split.value(), 150.0);
        #[cfg(feature = "decimal")]
        assert_eq!(split.value(), Decimal::new(150, 0));

        assert_eq!(split.quantity_num, 15000);
        assert_eq!(split.quantity_denom, 100);

        #[cfg(not(feature = "decimal"))]
        assert_eq!(split.quantity(), 150.0);
        #[cfg(feature = "decimal")]
        assert_eq!(split.quantity(), Decimal::new(150, 0));

        assert_eq!(split.lot_guid, None);
    }
    #[tokio::test]
    async fn transaction() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let split = book
            .splits()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
            .unwrap();
        let transaction = split.transaction().await.unwrap();
        assert_eq!(transaction.description.as_ref().unwrap(), "income 1");
    }

    #[tokio::test]
    async fn account() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let split = book
            .splits()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
            .unwrap();
        let account = split.account().await.unwrap();
        assert_eq!(account.name, "Cash");
    }
}

mod transaction {
    use super::*;
    #[tokio::test]
    async fn property() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let transaction = book
            .transactions()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
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

    #[tokio::test]
    async fn currency() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let transaction = book
            .transactions()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
            .unwrap();
        let currency = transaction.currency().await.unwrap();
        assert_eq!(currency.fullname.as_ref().unwrap(), "Euro");
    }

    #[tokio::test]
    async fn splits() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let transaction = book
            .transactions()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
            .unwrap();
        let splits = transaction.splits().await.unwrap();
        assert_eq!(splits.len(), 2);
    }
}

mod price {
    use super::*;
    #[tokio::test]
    async fn property() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let price = book
            .prices()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
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

        #[cfg(not(feature = "decimal"))]
        assert_eq!(price.value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(price.value(), Decimal::new(15, 1));
    }

    #[tokio::test]
    async fn commodity() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let price = book
            .prices()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
            .unwrap();
        let commodity = price.commodity().await.unwrap();
        assert_eq!(commodity.fullname.as_ref().unwrap(), "Andorran Franc");
    }

    #[tokio::test]
    async fn currency() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let price = book
            .prices()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
            .unwrap();
        let currency = price.currency().await.unwrap();
        assert_eq!(currency.fullname.as_ref().unwrap(), "UAE Dirham");
    }
}

mod commodity {
    use super::*;
    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;

    #[tokio::test]
    async fn property() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let commodity = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
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

    #[tokio::test]
    async fn accounts() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let commodity = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .unwrap();
        let accounts = commodity.accounts().await.unwrap();
        assert_eq!(accounts.len(), 14);
    }

    #[tokio::test]
    async fn transactions() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let commodity = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .unwrap();
        let transactions = commodity.transactions().await.unwrap();
        assert_eq!(transactions.len(), 11);
    }

    #[tokio::test]
    async fn as_commodity_prices() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let commodity = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .unwrap();
        let prices = commodity.as_commodity_prices().await.unwrap();
        assert_eq!(prices.len(), 1);
    }

    #[tokio::test]
    async fn as_currency_prices() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let commodity = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .unwrap();
        let prices = commodity.as_currency_prices().await.unwrap();
        assert_eq!(prices.len(), 2);
    }

    #[tokio::test]
    async fn as_commodity_or_currency_prices() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let commodity = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .unwrap();
        let prices = commodity.as_commodity_or_currency_prices().await.unwrap();
        assert_eq!(prices.len(), 3);
    }

    #[tokio::test]
    async fn rate_direct() {
        // ADF => AED
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let commodity = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "d821d6776fde9f7c2d01b67876406fd3")
            .unwrap();
        let currency = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
            .unwrap();

        let rate = commodity.sell(&currency).await.unwrap();
        #[cfg(not(feature = "decimal"))]
        assert_eq!(rate, 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(rate, Decimal::new(15, 1));

        let rate = currency.buy(&commodity).await.unwrap();
        #[cfg(not(feature = "decimal"))]
        assert_eq!(rate, 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(rate, Decimal::new(15, 1));

        // AED => EUR
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let commodity = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
            .unwrap();
        let currency = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "346629655191dcf59a7e2c2a85b70f69")
            .unwrap();

        let rate = commodity.sell(&currency).await.unwrap();
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, rate, 9.0 / 10.0);
        #[cfg(feature = "decimal")]
        assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));

        let rate = currency.buy(&commodity).await.unwrap();
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, rate, 9.0 / 10.0);
        #[cfg(feature = "decimal")]
        assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));
    }

    #[tokio::test]
    async fn rate_indirect() {
        let book = PostgreSQLBook::new(URI).await.unwrap();
        let commodity = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "1e5d65e2726a5d4595741cb204992991")
            .unwrap();
        let currency = book
            .commodities()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "5f586908098232e67edb1371408bfaa8")
            .unwrap();

        let rate = commodity.sell(&currency).await.unwrap();
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, rate, 7.0 / 5.0 * 10.0 / 9.0);
        #[cfg(feature = "decimal")]
        assert_eq!(
            rate,
            (Decimal::new(7, 0) / Decimal::new(5, 0)) * (Decimal::new(10, 0) / Decimal::new(9, 0)),
        );
    }
}
