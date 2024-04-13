#[cfg(feature = "mysql")]
pub(crate) mod mysql;
#[cfg(feature = "postgresql")]
pub(crate) mod postgresql;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;
#[cfg(feature = "xml")]
pub(crate) mod xml;

use chrono::NaiveDateTime;

use crate::error::Error;

pub trait Query:
    Clone + Sync + Send + AccountQ + CommodityQ + PriceQ + SplitQ + TransactionQ
{
    fn accounts(&self) -> impl std::future::Future<Output = Result<Vec<Self::A>, Error>> + Send {
        async { AccountQ::all(self).await }
    }
    fn accounts_contains_name_ignore_case(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::A>, Error>> + Send {
        async { AccountQ::contains_name_ignore_case(self, name).await }
    }
    fn splits(&self) -> impl std::future::Future<Output = Result<Vec<Self::S>, Error>> + Send {
        async { SplitQ::all(self).await }
    }
    fn transactions(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Self::T>, Error>> + Send {
        async { TransactionQ::all(self).await }
    }
    fn prices(&self) -> impl std::future::Future<Output = Result<Vec<Self::P>, Error>> + Send {
        async { PriceQ::all(self).await }
    }
    fn currencies(&self) -> impl std::future::Future<Output = Result<Vec<Self::C>, Error>> + Send {
        async { CommodityQ::namespace(self, "CURRENCY").await }
    }
    fn commodities(&self) -> impl std::future::Future<Output = Result<Vec<Self::C>, Error>> + Send {
        async { CommodityQ::all(self).await }
    }
}

pub trait AccountQ {
    type A: AccountT;

    fn all(&self) -> impl std::future::Future<Output = Result<Vec<Self::A>, Error>> + Send;
    fn guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::A>, Error>> + Send;
    fn commodity_guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::A>, Error>> + Send;
    fn parent_guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::A>, Error>> + Send;
    fn name(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::A>, Error>> + Send;
    fn contains_name_ignore_case(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::A>, Error>> + Send;
}

pub trait CommodityQ {
    type C: CommodityT;

    fn all(&self) -> impl std::future::Future<Output = Result<Vec<Self::C>, Error>> + Send;
    fn guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::C>, Error>> + Send;
    fn namespace(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::C>, Error>> + Send;
}

pub trait PriceQ {
    type P: PriceT;

    fn all(&self) -> impl std::future::Future<Output = Result<Vec<Self::P>, Error>> + Send;
    fn guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::P>, Error>> + Send;
    fn commodity_guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::P>, Error>> + Send;
    fn currency_guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::P>, Error>> + Send;
    fn commodity_or_currency_guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::P>, Error>> + Send;
}

pub trait SplitQ {
    type S: SplitT;

    fn all(&self) -> impl std::future::Future<Output = Result<Vec<Self::S>, Error>> + Send;
    fn guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::S>, Error>> + Send;
    fn account_guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::S>, Error>> + Send;
    fn tx_guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::S>, Error>> + Send;
}

pub trait TransactionQ {
    type T: TransactionT;

    fn all(&self) -> impl std::future::Future<Output = Result<Vec<Self::T>, Error>> + Send;
    fn guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::T>, Error>> + Send;
    fn currency_guid(
        &self,
        guid: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Self::T>, Error>> + Send;
}

pub trait AccountT {
    fn guid(&self) -> String;
    fn name(&self) -> String;
    fn account_type(&self) -> String;
    fn commodity_guid(&self) -> String;
    fn commodity_scu(&self) -> i64;
    fn non_std_scu(&self) -> bool;
    fn parent_guid(&self) -> String;
    fn code(&self) -> String;
    fn description(&self) -> String;
    fn hidden(&self) -> bool;
    fn placeholder(&self) -> bool;
}
pub trait CommodityT {
    fn guid(&self) -> String;
    fn namespace(&self) -> String;
    fn mnemonic(&self) -> String;
    fn fullname(&self) -> String;
    fn cusip(&self) -> String;
    fn fraction(&self) -> i64;
    fn quote_flag(&self) -> bool;
    fn quote_source(&self) -> String;
    fn quote_tz(&self) -> String;
}
pub trait PriceT {
    fn guid(&self) -> String;
    fn commodity_guid(&self) -> String;
    fn currency_guid(&self) -> String;
    fn datetime(&self) -> NaiveDateTime;
    fn source(&self) -> String;
    fn r#type(&self) -> String;
    fn value(&self) -> crate::Num;
}
pub trait SplitT {
    fn guid(&self) -> String;
    fn tx_guid(&self) -> String;
    fn account_guid(&self) -> String;
    fn memo(&self) -> String;
    fn action(&self) -> String;
    fn reconcile_state(&self) -> bool;
    fn reconcile_datetime(&self) -> Option<NaiveDateTime>;
    fn lot_guid(&self) -> String;
    fn value(&self) -> crate::Num;
    fn quantity(&self) -> crate::Num;
}
pub trait TransactionT {
    fn guid(&self) -> String;
    fn currency_guid(&self) -> String;
    fn num(&self) -> String;
    fn post_datetime(&self) -> NaiveDateTime;
    fn enter_datetime(&self) -> NaiveDateTime;
    fn description(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;
    use tokio::sync::OnceCell;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        use crate::SQLiteQuery;

        static Q: OnceCell<SQLiteQuery> = OnceCell::const_new();
        async fn setup() -> &'static SQLiteQuery {
            Q.get_or_init(|| async {
                let uri: &str = &format!(
                    "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
                    env!("CARGO_MANIFEST_DIR")
                );

                println!("work_dir: {:?}", std::env::current_dir());
                SQLiteQuery::new(&format!("{uri}?mode=ro")).await.unwrap()
            })
            .await
        }

        mod query {
            use super::*;

            use pretty_assertions::assert_eq;

            #[tokio::test]
            async fn test_accounts() {
                let query = setup().await;
                let result = query.accounts().await.unwrap();
                assert_eq!(result.len(), 21);
            }

            #[tokio::test]
            async fn test_accounts_contains_name() {
                let query = setup().await;
                let result = query
                    .accounts_contains_name_ignore_case("aS")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }

            #[tokio::test]
            async fn test_splits() {
                let query = setup().await;
                let result = query.splits().await.unwrap();
                assert_eq!(result.len(), 25);
            }

            #[tokio::test]
            async fn test_transactions() {
                let query = setup().await;
                let result = query.transactions().await.unwrap();
                assert_eq!(result.len(), 11);
            }

            #[tokio::test]
            async fn test_prices() {
                let query = setup().await;
                let result = query.prices().await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_commodities() {
                let query = setup().await;
                let result = query.commodities().await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_currencies() {
                let query = setup().await;
                let result = query.currencies().await.unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod account_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = AccountQ::all(query).await.unwrap();
                assert_eq!(result.len(), 21);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = AccountQ::guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();
                assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
            }

            #[tokio::test]
            async fn test_commodity_guid() {
                let query = setup().await;
                let result = AccountQ::commodity_guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();
                assert!(
                    result
                        .iter()
                        .map(|x| &x.guid)
                        .any(|guid| guid == "fcd795021c976ba75621ec39e75f6214"),
                    "result does not contains fcd795021c976ba75621ec39e75f6214"
                );
            }

            #[tokio::test]
            async fn test_parent_guid() {
                let query = setup().await;
                let result = AccountQ::parent_guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();
                assert_eq!(result[0].guid, "3bc319753945b6dba3e1928abed49e35");
            }

            #[tokio::test]
            async fn test_name() {
                let query = setup().await;
                let result = AccountQ::name(query, "Asset").await.unwrap();
                assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
            }

            #[tokio::test]
            async fn test_contains_name_ignore_case() {
                let query = setup().await;
                let result = AccountQ::contains_name_ignore_case(query, "aS")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }
        }
        mod commodity_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = CommodityQ::all(query).await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = CommodityQ::guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();
                assert_eq!(result[0].fullname.as_ref().unwrap(), "Euro");
            }

            #[tokio::test]
            async fn test_namespace() {
                let query = setup().await;
                let result = CommodityQ::namespace(query, "CURRENCY").await.unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod price_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = PriceQ::all(query).await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = PriceQ::guid(query, "0d6684f44fb018e882de76094ed9c433")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn commodity_guid() {
                let query = setup().await;
                let result = PriceQ::commodity_guid(query, "d821d6776fde9f7c2d01b67876406fd3")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn currency_guid() {
                let query = setup().await;
                let result = PriceQ::currency_guid(query, "5f586908098232e67edb1371408bfaa8")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn commodity_or_currency_guid() {
                let query = setup().await;
                let result =
                    PriceQ::commodity_or_currency_guid(query, "5f586908098232e67edb1371408bfaa8")
                        .await
                        .unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod split_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = SplitQ::all(query).await.unwrap();
                assert_eq!(result.len(), 25);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = SplitQ::guid(query, "de832fe97e37811a7fff7e28b3a43425")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(150, 0));
            }

            #[tokio::test]
            async fn test_account_guid() {
                let query = setup().await;
                let result = SplitQ::account_guid(query, "93fc043c3062aaa1297b30e543d2cd0d")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }

            #[tokio::test]
            async fn test_tx_guid() {
                let query = setup().await;
                let result = SplitQ::tx_guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 2);
            }
        }
        mod transaction_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = TransactionQ::all(query).await.unwrap();
                assert_eq!(result.len(), 11);
            }

            #[tokio::test]
            async fn test_by_guid() {
                let query = setup().await;
                let result = TransactionQ::guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();

                assert_eq!(
                    result[0].post_date.unwrap(),
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );

                assert_eq!(
                    result[0].enter_date.unwrap(),
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
            }

            #[tokio::test]
            async fn test_currency_guid() {
                let query = setup().await;
                let result = TransactionQ::currency_guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();

                assert_eq!(result.len(), 11);
            }
        }
        mod account_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = AccountQ::guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "fcd795021c976ba75621ec39e75f6214");
                assert_eq!(result.name(), "Asset");
                assert_eq!(result.account_type(), "ASSET");
                assert_eq!(result.commodity_guid(), "346629655191dcf59a7e2c2a85b70f69");
                assert_eq!(result.commodity_scu(), 100);
                assert!(!result.non_std_scu());
                assert_eq!(result.parent_guid(), "00622dda21937b29e494179de5013f82");
                assert_eq!(result.code(), "");
                assert_eq!(result.description(), "");
                assert!(!result.hidden());
                assert!(result.placeholder());
            }
        }
        mod commodity_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = CommodityQ::guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "346629655191dcf59a7e2c2a85b70f69");
                assert_eq!(result.namespace(), "CURRENCY");
                assert_eq!(result.mnemonic(), "EUR");
                assert_eq!(result.fullname(), "Euro");
                assert_eq!(result.cusip(), "978");
                assert_eq!(result.fraction(), 100);
                assert!(result.quote_flag());
                assert_eq!(result.quote_source(), "currency");
                assert_eq!(result.quote_tz(), "");
            }
        }
        mod price_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = PriceQ::guid(query, "0d6684f44fb018e882de76094ed9c433")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "0d6684f44fb018e882de76094ed9c433");
                assert_eq!(result.commodity_guid(), "d821d6776fde9f7c2d01b67876406fd3");
                assert_eq!(result.currency_guid(), "5f586908098232e67edb1371408bfaa8");
                assert_eq!(
                    result.datetime(),
                    NaiveDateTime::parse_from_str("2018-02-20 23:00:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(result.source(), "user:price-editor");
                assert_eq!(result.r#type(), "unknown");
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result.value(), Decimal::new(15, 1));
            }
        }
        mod split_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = SplitQ::guid(query, "de832fe97e37811a7fff7e28b3a43425")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "de832fe97e37811a7fff7e28b3a43425");
                assert_eq!(result.tx_guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
                assert_eq!(result.account_guid(), "93fc043c3062aaa1297b30e543d2cd0d");
                assert_eq!(result.memo(), "");
                assert_eq!(result.action(), "");
                assert!(!result.reconcile_state());
                assert_eq!(result.reconcile_datetime(), None);
                assert_eq!(result.lot_guid(), "");
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.value(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result.value(), Decimal::new(150, 0));
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.quantity(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result.quantity(), Decimal::new(150, 0));
            }
        }
        mod transaction_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = TransactionQ::guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
                assert_eq!(result.currency_guid(), "346629655191dcf59a7e2c2a85b70f69");
                assert_eq!(result.num(), "");
                assert_eq!(
                    result.post_datetime(),
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(
                    result.enter_datetime(),
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(result.description(), "income 1");
            }
        }
    }

    #[cfg(feature = "mysql")]
    mod mysql {
        use super::*;

        use crate::MySQLQuery;

        static Q: OnceCell<MySQLQuery> = OnceCell::const_new();
        async fn setup() -> &'static MySQLQuery {
            Q.get_or_init(|| async {
                let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
                MySQLQuery::new(uri).await.unwrap()
            })
            .await
        }

        mod query {
            use super::*;

            use pretty_assertions::assert_eq;

            #[tokio::test]
            async fn test_accounts() {
                let query = setup().await;
                let result = query.accounts().await.unwrap();
                assert_eq!(result.len(), 21);
            }

            #[tokio::test]
            async fn test_accounts_contains_name() {
                let query = setup().await;
                let result = query
                    .accounts_contains_name_ignore_case("aS")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }

            #[tokio::test]
            async fn test_splits() {
                let query = setup().await;
                let result = query.splits().await.unwrap();
                assert_eq!(result.len(), 25);
            }

            #[tokio::test]
            async fn test_transactions() {
                let query = setup().await;
                let result = query.transactions().await.unwrap();
                assert_eq!(result.len(), 11);
            }

            #[tokio::test]
            async fn test_prices() {
                let query = setup().await;
                let result = query.prices().await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_commodities() {
                let query = setup().await;
                let result = query.commodities().await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_currencies() {
                let query = setup().await;
                let result = query.currencies().await.unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod account_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = AccountQ::all(query).await.unwrap();
                assert_eq!(result.len(), 21);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = AccountQ::guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();
                assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
            }

            #[tokio::test]
            async fn test_commodity_guid() {
                let query = setup().await;
                let result = AccountQ::commodity_guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();
                assert!(
                    result
                        .iter()
                        .map(|x| &x.guid)
                        .any(|guid| guid == "fcd795021c976ba75621ec39e75f6214"),
                    "result does not contains fcd795021c976ba75621ec39e75f6214"
                );
            }

            #[tokio::test]
            async fn test_parent_guid() {
                let query = setup().await;
                let result = AccountQ::parent_guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();
                assert!(
                    result
                        .iter()
                        .map(|x| &x.guid)
                        .any(|guid| guid == "3bc319753945b6dba3e1928abed49e35"),
                    "result does not contains 3bc319753945b6dba3e1928abed49e35"
                );
            }

            #[tokio::test]
            async fn test_name() {
                let query = setup().await;
                let result = AccountQ::name(query, "Asset").await.unwrap();
                assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
            }

            #[tokio::test]
            async fn test_contains_name_ignore_case() {
                let query = setup().await;
                let result = AccountQ::contains_name_ignore_case(query, "aS")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }
        }
        mod commodity_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = CommodityQ::all(query).await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = CommodityQ::guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();
                assert_eq!(result[0].fullname.as_ref().unwrap(), "Euro");
            }

            #[tokio::test]
            async fn test_namespace() {
                let query = setup().await;
                let result = CommodityQ::namespace(query, "CURRENCY").await.unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod price_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = PriceQ::all(query).await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = PriceQ::guid(query, "0d6684f44fb018e882de76094ed9c433")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn commodity_guid() {
                let query = setup().await;
                let result = PriceQ::commodity_guid(query, "d821d6776fde9f7c2d01b67876406fd3")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn currency_guid() {
                let query = setup().await;
                let result = PriceQ::currency_guid(query, "5f586908098232e67edb1371408bfaa8")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn commodity_or_currency_guid() {
                let query = setup().await;
                let result =
                    PriceQ::commodity_or_currency_guid(query, "5f586908098232e67edb1371408bfaa8")
                        .await
                        .unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod split_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = SplitQ::all(query).await.unwrap();
                assert_eq!(result.len(), 25);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = SplitQ::guid(query, "de832fe97e37811a7fff7e28b3a43425")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(150, 0));
            }

            #[tokio::test]
            async fn test_account_guid() {
                let query = setup().await;
                let result = SplitQ::account_guid(query, "93fc043c3062aaa1297b30e543d2cd0d")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }

            #[tokio::test]
            async fn test_tx_guid() {
                let query = setup().await;
                let result = SplitQ::tx_guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 2);
            }
        }
        mod transaction_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = TransactionQ::all(query).await.unwrap();
                assert_eq!(result.len(), 11);
            }

            #[tokio::test]
            async fn test_by_guid() {
                let query = setup().await;
                let result = TransactionQ::guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();

                assert_eq!(
                    result[0].post_date.unwrap(),
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );

                assert_eq!(
                    result[0].enter_date.unwrap(),
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
            }

            #[tokio::test]
            async fn test_currency_guid() {
                let query = setup().await;
                let result = TransactionQ::currency_guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();

                assert_eq!(result.len(), 11);
            }
        }
        mod account_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = AccountQ::guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "fcd795021c976ba75621ec39e75f6214");
                assert_eq!(result.name(), "Asset");
                assert_eq!(result.account_type(), "ASSET");
                assert_eq!(result.commodity_guid(), "346629655191dcf59a7e2c2a85b70f69");
                assert_eq!(result.commodity_scu(), 100);
                assert!(!result.non_std_scu());
                assert_eq!(result.parent_guid(), "00622dda21937b29e494179de5013f82");
                assert_eq!(result.code(), "");
                assert_eq!(result.description(), "");
                assert!(!result.hidden());
                assert!(result.placeholder());
            }
        }
        mod commodity_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = CommodityQ::guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "346629655191dcf59a7e2c2a85b70f69");
                assert_eq!(result.namespace(), "CURRENCY");
                assert_eq!(result.mnemonic(), "EUR");
                assert_eq!(result.fullname(), "Euro");
                assert_eq!(result.cusip(), "978");
                assert_eq!(result.fraction(), 100);
                assert!(result.quote_flag());
                assert_eq!(result.quote_source(), "currency");
                assert_eq!(result.quote_tz(), "");
            }
        }
        mod price_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = PriceQ::guid(query, "0d6684f44fb018e882de76094ed9c433")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "0d6684f44fb018e882de76094ed9c433");
                assert_eq!(result.commodity_guid(), "d821d6776fde9f7c2d01b67876406fd3");
                assert_eq!(result.currency_guid(), "5f586908098232e67edb1371408bfaa8");
                assert_eq!(
                    result.datetime(),
                    NaiveDateTime::parse_from_str("2018-02-20 23:00:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(result.source(), "user:price-editor");
                assert_eq!(result.r#type(), "unknown");
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result.value(), Decimal::new(15, 1));
            }
        }
        mod split_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = SplitQ::guid(query, "de832fe97e37811a7fff7e28b3a43425")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "de832fe97e37811a7fff7e28b3a43425");
                assert_eq!(result.tx_guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
                assert_eq!(result.account_guid(), "93fc043c3062aaa1297b30e543d2cd0d");
                assert_eq!(result.memo(), "");
                assert_eq!(result.action(), "");
                assert!(!result.reconcile_state());
                assert_eq!(result.reconcile_datetime(), None);
                assert_eq!(result.lot_guid(), "");
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.value(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result.value(), Decimal::new(150, 0));
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.quantity(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result.quantity(), Decimal::new(150, 0));
            }
        }
        mod transaction_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = TransactionQ::guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
                assert_eq!(result.currency_guid(), "346629655191dcf59a7e2c2a85b70f69");
                assert_eq!(result.num(), "");
                assert_eq!(
                    result.post_datetime(),
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(
                    result.enter_datetime(),
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(result.description(), "income 1");
            }
        }
    }

    #[cfg(feature = "postgresql")]
    mod postgresql {
        use super::*;

        use crate::PostgreSQLQuery;

        static Q: OnceCell<PostgreSQLQuery> = OnceCell::const_new();
        async fn setup() -> &'static PostgreSQLQuery {
            Q.get_or_init(|| async {
                let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
                PostgreSQLQuery::new(uri).await.unwrap()
            })
            .await
        }

        mod query {
            use super::*;

            use pretty_assertions::assert_eq;

            #[tokio::test]
            async fn test_accounts() {
                let query = setup().await;
                let result = query.accounts().await.unwrap();
                assert_eq!(result.len(), 21);
            }

            #[tokio::test]
            async fn test_accounts_contains_name() {
                let query = setup().await;
                let result = query
                    .accounts_contains_name_ignore_case("aS")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }

            #[tokio::test]
            async fn test_splits() {
                let query = setup().await;
                let result = query.splits().await.unwrap();
                assert_eq!(result.len(), 25);
            }

            #[tokio::test]
            async fn test_transactions() {
                let query = setup().await;
                let result = query.transactions().await.unwrap();
                assert_eq!(result.len(), 11);
            }

            #[tokio::test]
            async fn test_prices() {
                let query = setup().await;
                let result = query.prices().await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_commodities() {
                let query = setup().await;
                let result = query.commodities().await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_currencies() {
                let query = setup().await;
                let result = query.currencies().await.unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod account_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = AccountQ::all(query).await.unwrap();
                assert_eq!(result.len(), 21);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = AccountQ::guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();
                assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
            }

            #[tokio::test]
            async fn test_commodity_guid() {
                let query = setup().await;
                let result = AccountQ::commodity_guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();
                assert!(
                    result
                        .iter()
                        .map(|x| &x.guid)
                        .any(|guid| guid == "fcd795021c976ba75621ec39e75f6214"),
                    "result does not contains fcd795021c976ba75621ec39e75f6214"
                );
            }

            #[tokio::test]
            async fn test_parent_guid() {
                let query = setup().await;
                let result = AccountQ::parent_guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();
                assert!(
                    result
                        .iter()
                        .map(|x| &x.guid)
                        .any(|guid| guid == "3bc319753945b6dba3e1928abed49e35"),
                    "result does not contains 3bc319753945b6dba3e1928abed49e35"
                );
            }

            #[tokio::test]
            async fn test_name() {
                let query = setup().await;
                let result = AccountQ::name(query, "Asset").await.unwrap();
                assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
            }

            #[tokio::test]
            async fn test_contains_name_ignore_case() {
                let query = setup().await;
                let result = AccountQ::contains_name_ignore_case(query, "aS")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }
        }
        mod commodity_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = CommodityQ::all(query).await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = CommodityQ::guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();
                assert_eq!(result[0].fullname.as_ref().unwrap(), "Euro");
            }

            #[tokio::test]
            async fn test_namespace() {
                let query = setup().await;
                let result = CommodityQ::namespace(query, "CURRENCY").await.unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod price_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = PriceQ::all(query).await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = PriceQ::guid(query, "0d6684f44fb018e882de76094ed9c433")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn commodity_guid() {
                let query = setup().await;
                let result = PriceQ::commodity_guid(query, "d821d6776fde9f7c2d01b67876406fd3")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn currency_guid() {
                let query = setup().await;
                let result = PriceQ::currency_guid(query, "5f586908098232e67edb1371408bfaa8")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn commodity_or_currency_guid() {
                let query = setup().await;
                let result =
                    PriceQ::commodity_or_currency_guid(query, "5f586908098232e67edb1371408bfaa8")
                        .await
                        .unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod split_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = SplitQ::all(query).await.unwrap();
                assert_eq!(result.len(), 25);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = SplitQ::guid(query, "de832fe97e37811a7fff7e28b3a43425")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(150, 0));
            }

            #[tokio::test]
            async fn test_account_guid() {
                let query = setup().await;
                let result = SplitQ::account_guid(query, "93fc043c3062aaa1297b30e543d2cd0d")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }

            #[tokio::test]
            async fn test_tx_guid() {
                let query = setup().await;
                let result = SplitQ::tx_guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 2);
            }
        }
        mod transaction_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = TransactionQ::all(query).await.unwrap();
                assert_eq!(result.len(), 11);
            }

            #[tokio::test]
            async fn test_by_guid() {
                let query = setup().await;
                let result = TransactionQ::guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();

                assert_eq!(
                    result[0].post_date.unwrap(),
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );

                assert_eq!(
                    result[0].enter_date.unwrap(),
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
            }

            #[tokio::test]
            async fn test_currency_guid() {
                let query = setup().await;
                let result = TransactionQ::currency_guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();

                assert_eq!(result.len(), 11);
            }
        }
        mod account_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = AccountQ::guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "fcd795021c976ba75621ec39e75f6214");
                assert_eq!(result.name(), "Asset");
                assert_eq!(result.account_type(), "ASSET");
                assert_eq!(result.commodity_guid(), "346629655191dcf59a7e2c2a85b70f69");
                assert_eq!(result.commodity_scu(), 100);
                assert!(!result.non_std_scu());
                assert_eq!(result.parent_guid(), "00622dda21937b29e494179de5013f82");
                assert_eq!(result.code(), "");
                assert_eq!(result.description(), "");
                assert!(!result.hidden());
                assert!(result.placeholder());
            }
        }
        mod commodity_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = CommodityQ::guid(query, "346629655191dcf59a7e2c2a85b70f69")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "346629655191dcf59a7e2c2a85b70f69");
                assert_eq!(result.namespace(), "CURRENCY");
                assert_eq!(result.mnemonic(), "EUR");
                assert_eq!(result.fullname(), "Euro");
                assert_eq!(result.cusip(), "978");
                assert_eq!(result.fraction(), 100);
                assert!(result.quote_flag());
                assert_eq!(result.quote_source(), "currency");
                assert_eq!(result.quote_tz(), "");
            }
        }
        mod price_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = PriceQ::guid(query, "0d6684f44fb018e882de76094ed9c433")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "0d6684f44fb018e882de76094ed9c433");
                assert_eq!(result.commodity_guid(), "d821d6776fde9f7c2d01b67876406fd3");
                assert_eq!(result.currency_guid(), "5f586908098232e67edb1371408bfaa8");
                assert_eq!(
                    result.datetime(),
                    NaiveDateTime::parse_from_str("2018-02-20 23:00:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(result.source(), "user:price-editor");
                assert_eq!(result.r#type(), "unknown");
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result.value(), Decimal::new(15, 1));
            }
        }
        mod split_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = SplitQ::guid(query, "de832fe97e37811a7fff7e28b3a43425")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "de832fe97e37811a7fff7e28b3a43425");
                assert_eq!(result.tx_guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
                assert_eq!(result.account_guid(), "93fc043c3062aaa1297b30e543d2cd0d");
                assert_eq!(result.memo(), "");
                assert_eq!(result.action(), "");
                assert!(!result.reconcile_state());
                assert_eq!(result.reconcile_datetime(), None);
                assert_eq!(result.lot_guid(), "");
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.value(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result.value(), Decimal::new(150, 0));
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.quantity(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result.quantity(), Decimal::new(150, 0));
            }
        }
        mod transaction_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = TransactionQ::guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
                assert_eq!(result.currency_guid(), "346629655191dcf59a7e2c2a85b70f69");
                assert_eq!(result.num(), "");
                assert_eq!(
                    result.post_datetime(),
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(
                    result.enter_datetime(),
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(result.description(), "income 1");
            }
        }
    }

    #[cfg(feature = "xml")]
    mod xml {
        use super::*;

        use crate::XMLQuery;

        static Q: OnceCell<XMLQuery> = OnceCell::const_new();
        async fn setup() -> &'static XMLQuery {
            Q.get_or_init(|| async {
                let path: &str = &format!(
                    "{}/tests/db/xml/complex_sample.gnucash",
                    env!("CARGO_MANIFEST_DIR")
                );

                println!("work_dir: {:?}", std::env::current_dir());
                XMLQuery::new(path).unwrap()
            })
            .await
        }

        mod query {
            use super::*;

            use pretty_assertions::assert_eq;

            #[tokio::test]
            async fn test_accounts() {
                let query = setup().await;
                let result = query.accounts().await.unwrap();
                assert_eq!(result.len(), 20);
            }

            #[tokio::test]
            async fn test_accounts_contains_name() {
                let query = setup().await;
                let result = query
                    .accounts_contains_name_ignore_case("aS")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }

            #[tokio::test]
            async fn test_splits() {
                let query = setup().await;
                let result = query.splits().await.unwrap();
                assert_eq!(result.len(), 25);
            }

            #[tokio::test]
            async fn test_transactions() {
                let query = setup().await;
                let result = query.transactions().await.unwrap();
                assert_eq!(result.len(), 11);
            }

            #[tokio::test]
            async fn test_prices() {
                let query = setup().await;
                let result = query.prices().await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_commodities() {
                let query = setup().await;
                let result = query.commodities().await.unwrap();
                assert_eq!(result.len(), 6);
            }

            #[tokio::test]
            async fn test_currencies() {
                let query = setup().await;
                let result = query.currencies().await.unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod account_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = AccountQ::all(query).await.unwrap();
                assert_eq!(result.len(), 20);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = AccountQ::guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();
                assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
            }

            #[tokio::test]
            async fn test_commodity_guid() {
                let query = setup().await;
                let result = AccountQ::commodity_guid(query, "EUR").await.unwrap();
                assert!(
                    result
                        .iter()
                        .map(|x| &x.guid)
                        .any(|guid| guid == "fcd795021c976ba75621ec39e75f6214"),
                    "result does not contains fcd795021c976ba75621ec39e75f6214"
                );
            }

            #[tokio::test]
            async fn test_parent_guid() {
                let query = setup().await;
                let result = AccountQ::parent_guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();
                assert!(
                    result
                        .iter()
                        .map(|x| &x.guid)
                        .any(|guid| guid == "3bc319753945b6dba3e1928abed49e35"),
                    "result does not contains 3bc319753945b6dba3e1928abed49e35"
                );
            }

            #[tokio::test]
            async fn test_name() {
                let query = setup().await;
                let result = AccountQ::name(query, "Asset").await.unwrap();
                assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
            }

            #[tokio::test]
            async fn test_contains_name_ignore_case() {
                let query = setup().await;
                let result = AccountQ::contains_name_ignore_case(query, "aS")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }
        }
        mod commodity_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = CommodityQ::all(query).await.unwrap();
                assert_eq!(result.len(), 6);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = CommodityQ::guid(query, "FOO").await.unwrap();
                assert_eq!(result[0].fullname.as_ref().unwrap(), "Foo Inc");
            }

            #[tokio::test]
            async fn test_namespace() {
                let query = setup().await;
                let result = CommodityQ::namespace(query, "CURRENCY").await.unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod price_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = PriceQ::all(query).await.unwrap();
                assert_eq!(result.len(), 5);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = PriceQ::guid(query, "0d6684f44fb018e882de76094ed9c433")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn commodity_guid() {
                let query = setup().await;
                let result = PriceQ::commodity_guid(query, "ADF").await.unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn currency_guid() {
                let query = setup().await;
                let result = PriceQ::currency_guid(query, "AED").await.unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(15, 1));
            }

            #[tokio::test]
            async fn commodity_or_currency_guid() {
                let query = setup().await;
                let result = PriceQ::commodity_or_currency_guid(query, "AED")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 4);
            }
        }
        mod split_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = SplitQ::all(query).await.unwrap();
                assert_eq!(result.len(), 25);
            }

            #[tokio::test]
            async fn test_guid() {
                let query = setup().await;
                let result = SplitQ::guid(query, "de832fe97e37811a7fff7e28b3a43425")
                    .await
                    .unwrap();

                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result[0].value(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result[0].value(), Decimal::new(150, 0));
            }

            #[tokio::test]
            async fn test_account_guid() {
                let query = setup().await;
                let result = SplitQ::account_guid(query, "93fc043c3062aaa1297b30e543d2cd0d")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 3);
            }

            #[tokio::test]
            async fn test_tx_guid() {
                let query = setup().await;
                let result = SplitQ::tx_guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();
                assert_eq!(result.len(), 2);
            }
        }
        mod transaction_q {
            use super::*;

            #[tokio::test]
            async fn test_all() {
                let query = setup().await;
                let result = TransactionQ::all(query).await.unwrap();
                assert_eq!(result.len(), 11);
            }

            #[tokio::test]
            async fn test_by_guid() {
                let query = setup().await;
                let result = TransactionQ::guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();

                assert_eq!(
                    result[0].post_date.unwrap(),
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );

                assert_eq!(
                    result[0].enter_date.unwrap(),
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
            }

            #[tokio::test]
            async fn test_currency_guid() {
                let query = setup().await;
                let result = TransactionQ::currency_guid(query, "EUR").await.unwrap();

                assert_eq!(result.len(), 11);
            }
        }
        mod account_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = AccountQ::guid(query, "fcd795021c976ba75621ec39e75f6214")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "fcd795021c976ba75621ec39e75f6214");
                assert_eq!(result.name(), "Asset");
                assert_eq!(result.account_type(), "ASSET");
                assert_eq!(result.commodity_guid(), "EUR");
                assert_eq!(result.commodity_scu(), 100);
                assert!(!result.non_std_scu());
                assert_eq!(result.parent_guid(), "00622dda21937b29e494179de5013f82");
                assert_eq!(result.code(), "");
                assert_eq!(result.description(), "");
                assert!(!result.hidden());
                assert!(result.placeholder());
            }
        }
        mod commodity_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = CommodityQ::guid(query, "EUR").await.unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "EUR");
                assert_eq!(result.namespace(), "CURRENCY");
                assert_eq!(result.mnemonic(), "EUR");
                assert_eq!(result.fullname(), "");
                assert_eq!(result.cusip(), "");
                assert_eq!(result.fraction(), 100);
                assert!(result.quote_flag());
                assert_eq!(result.quote_source(), "currency");
                assert_eq!(result.quote_tz(), "");
            }
        }
        mod price_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = PriceQ::guid(query, "0d6684f44fb018e882de76094ed9c433")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "0d6684f44fb018e882de76094ed9c433");
                assert_eq!(result.commodity_guid(), "ADF");
                assert_eq!(result.currency_guid(), "AED");
                assert_eq!(
                    result.datetime(),
                    NaiveDateTime::parse_from_str("2018-02-20 23:00:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(result.source(), "user:price-editor");
                assert_eq!(result.r#type(), "unknown");
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.value(), 1.5);
                #[cfg(feature = "decimal")]
                assert_eq!(result.value(), Decimal::new(15, 1));
            }
        }
        mod split_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = SplitQ::guid(query, "de832fe97e37811a7fff7e28b3a43425")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "de832fe97e37811a7fff7e28b3a43425");
                assert_eq!(result.tx_guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
                assert_eq!(result.account_guid(), "93fc043c3062aaa1297b30e543d2cd0d");
                assert_eq!(result.memo(), "");
                assert_eq!(result.action(), "");
                assert!(!result.reconcile_state());
                assert_eq!(result.reconcile_datetime(), None);
                assert_eq!(result.lot_guid(), "");
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.value(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result.value(), Decimal::new(150, 0));
                #[cfg(not(feature = "decimal"))]
                assert_approx_eq!(f64, result.quantity(), 150.0);
                #[cfg(feature = "decimal")]
                assert_eq!(result.quantity(), Decimal::new(150, 0));
            }
        }
        mod transaction_t {
            use super::*;

            #[tokio::test]
            async fn test_trait_fn() {
                let query = setup().await;
                let result = TransactionQ::guid(query, "6c8876003c4a6026e38e3afb67d6f2b1")
                    .await
                    .unwrap();

                let result = &result[0];
                assert_eq!(result.guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
                assert_eq!(result.currency_guid(), "EUR");
                assert_eq!(result.num(), "");
                assert_eq!(
                    result.post_datetime(),
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(
                    result.enter_datetime(),
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                );
                assert_eq!(result.description(), "income 1");
            }
        }
    }
}
