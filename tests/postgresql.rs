use chrono::NaiveDateTime;
#[cfg(not(feature = "decimal"))]
use float_cmp::assert_approx_eq;
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;

use rucash::{Book, PostgreSQLQuery};

pub fn uri() -> String {
    "postgresql://user:secret@localhost:5432/complex_sample.gnucash".into()
}

mod book {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn new() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        Book::new(query).await.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn new_fail() {
        let query = PostgreSQLQuery::new("postgresql://complex_sample.gnucash")
            .await
            .unwrap();
        Book::new(query).await.unwrap();
    }

    #[tokio::test]
    async fn accounts() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let accounts = book.accounts().await.unwrap();
        assert_eq!(accounts.len(), 21);
    }

    #[tokio::test]
    async fn accounts_filter() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let accounts = book.accounts_contains_name_ignore_case("aS").await.unwrap();
        assert_eq!(accounts.len(), 3);
    }

    #[tokio::test]
    async fn account_contains_name_ignore_case() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let account = book
            .account_contains_name_ignore_case("NAS")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(account.name, "NASDAQ");
    }

    #[tokio::test]
    async fn splits() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let splits = book.splits().await.unwrap();
        assert_eq!(splits.len(), 25);
    }

    #[tokio::test]
    async fn transactions() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let transactions = book.transactions().await.unwrap();
        assert_eq!(transactions.len(), 11);
    }

    #[tokio::test]
    async fn prices() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let prices = book.prices().await.unwrap();
        assert_eq!(prices.len(), 5);
    }

    #[tokio::test]
    async fn commodities() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let commodities = book.commodities().await.unwrap();
        assert_eq!(commodities.len(), 5);
    }

    #[tokio::test]
    async fn currencies() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let currencies = book.currencies().await.unwrap();
        assert_eq!(currencies.len(), 4);
    }
}
mod account {
    use super::*;
    use pretty_assertions::assert_eq;
    #[tokio::test]
    async fn property() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let account = book
            .accounts()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "fcd795021c976ba75621ec39e75f6214")
            .unwrap();

        assert_eq!(account.guid, "fcd795021c976ba75621ec39e75f6214");
        assert_eq!(account.name, "Asset");
        assert_eq!(account.r#type, "ASSET");
        assert_eq!(account.commodity_guid, "346629655191dcf59a7e2c2a85b70f69");
        assert_eq!(account.commodity_scu, 100);
        assert_eq!(account.non_std_scu, false);
        assert_eq!(account.parent_guid, "00622dda21937b29e494179de5013f82");
        assert_eq!(account.code, "");
        assert_eq!(account.description, "");
        assert_eq!(account.hidden, false);
        assert_eq!(account.placeholder, true);
    }

    #[tokio::test]
    async fn balance() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let account = book
            .accounts()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.name == "Current")
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, account.balance(&book).await.unwrap(), 4590.0);
        #[cfg(feature = "decimal")]
        assert_eq!(account.balance(&book).await.unwrap(), Decimal::new(4590, 0));
    }
    #[tokio::test]
    async fn balance_diff_currency() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let account = book
            .accounts()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.name == "Asset")
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, account.balance(&book).await.unwrap(), 24695.3);
        #[cfg(feature = "decimal")]
        assert_eq!(
            account.balance(&book).await.unwrap(),
            Decimal::new(246953, 1)
        );
    }
    #[tokio::test]
    async fn splits() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let account = book
            .account_contains_name_ignore_case("Cash")
            .await
            .unwrap()
            .unwrap();
        let splits = account.splits().await.unwrap();
        assert_eq!(splits.len(), 3);
    }

    #[tokio::test]
    async fn parent() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let account = book
            .account_contains_name_ignore_case("Cash")
            .await
            .unwrap()
            .unwrap();
        let parent = account.parent().await.unwrap().unwrap();
        assert_eq!(parent.name, "Current");
    }

    #[tokio::test]
    async fn no_parent() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let account = book
            .account_contains_name_ignore_case("Root Account")
            .await
            .unwrap()
            .unwrap();
        let parent = account.parent().await.unwrap();
        assert!(parent.is_none());
    }

    #[tokio::test]
    async fn children() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let account = book
            .account_contains_name_ignore_case("Current")
            .await
            .unwrap()
            .unwrap();
        let children = account.children().await.unwrap();
        assert_eq!(children.len(), 3);
    }

    #[tokio::test]
    async fn commodity() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let account = book
            .account_contains_name_ignore_case("Cash")
            .await
            .unwrap()
            .unwrap();
        let commodity = account.commodity().await.unwrap();
        assert_eq!(commodity.mnemonic, "EUR");
    }
}

mod split {
    use super::*;
    use pretty_assertions::assert_eq;
    #[tokio::test]
    async fn property() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
        assert_eq!(split.reconcile_state, false);
        assert_eq!(split.reconcile_datetime, None);

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, split.value, 150.0);
        #[cfg(feature = "decimal")]
        assert_eq!(split.value, Decimal::new(150, 0));

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, split.quantity, 150.0);
        #[cfg(feature = "decimal")]
        assert_eq!(split.quantity, Decimal::new(150, 0));

        assert_eq!(split.lot_guid, "");
    }
    #[tokio::test]
    async fn transaction() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let split = book
            .splits()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
            .unwrap();
        let transaction = split.transaction().await.unwrap();
        assert_eq!(transaction.description, "income 1");
    }

    #[tokio::test]
    async fn account() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
    use pretty_assertions::assert_eq;
    #[tokio::test]
    async fn property() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
            transaction.post_datetime,
            NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(
            transaction.enter_datetime,
            NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(transaction.description, "income 1");
    }

    #[tokio::test]
    async fn currency() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let transaction = book
            .transactions()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "6c8876003c4a6026e38e3afb67d6f2b1")
            .unwrap();
        let currency = transaction.currency().await.unwrap();
        assert_eq!(currency.fullname, "Euro");
    }

    #[tokio::test]
    async fn splits() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
    use pretty_assertions::assert_eq;
    #[tokio::test]
    async fn property() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
            price.datetime,
            NaiveDateTime::parse_from_str("2018-02-20 23:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(price.source, "user:price-editor");
        assert_eq!(price.r#type, "unknown");

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, price.value, 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(price.value, Decimal::new(15, 1));
    }

    #[tokio::test]
    async fn commodity() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let price = book
            .prices()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
            .unwrap();
        let commodity = price.commodity().await.unwrap();
        assert_eq!(commodity.fullname, "Andorran Franc");
    }

    #[tokio::test]
    async fn currency() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
        let price = book
            .prices()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "0d6684f44fb018e882de76094ed9c433")
            .unwrap();
        let currency = price.currency().await.unwrap();
        assert_eq!(currency.fullname, "UAE Dirham");
    }
}

mod commodity {
    use super::*;

    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn property() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
        assert_eq!(commodity.fullname, "Euro");
        assert_eq!(commodity.cusip, "978");
        assert_eq!(commodity.fraction, 100);
        assert_eq!(commodity.quote_flag, true);
        assert_eq!(commodity.quote_source, "currency");
        assert_eq!(commodity.quote_tz, "");
    }

    #[tokio::test]
    async fn accounts() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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

        let rate = commodity.sell(&currency, &book).await.unwrap();
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, rate, 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(rate, Decimal::new(15, 1));

        let rate = currency.buy(&commodity, &book).await.unwrap();
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, rate, 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(rate, Decimal::new(15, 1));

        // AED => EUR
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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

        let rate = commodity.sell(&currency, &book).await.unwrap();
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, rate, 9.0 / 10.0);
        #[cfg(feature = "decimal")]
        assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));

        let rate = currency.buy(&commodity, &book).await.unwrap();
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, rate, 9.0 / 10.0);
        #[cfg(feature = "decimal")]
        assert_eq!(rate, Decimal::new(9, 0) / Decimal::new(10, 0));
    }

    #[tokio::test]
    async fn rate_indirect() {
        let query = PostgreSQLQuery::new(&uri()).await.unwrap();
        let book = Book::new(query).await.unwrap();
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

        let rate = commodity.sell(&currency, &book).await.unwrap();
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, rate, 7.0 / 5.0 * 10.0 / 9.0);
        #[cfg(feature = "decimal")]
        assert_eq!(
            rate,
            (Decimal::new(7, 0) / Decimal::new(5, 0)) * (Decimal::new(10, 0) / Decimal::new(9, 0)),
        );
    }
}
