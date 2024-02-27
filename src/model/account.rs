use futures::future::{BoxFuture, FutureExt};
use std::sync::Arc;

use crate::error::Error;
use crate::model::{Commodity, Split};
use crate::query::{AccountQ, AccountT, CommodityQ, Query, SplitQ};
use crate::Book;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Account<Q>
where
    Q: Query,
{
    query: Arc<Q>,

    pub guid: String,
    pub name: String,
    pub r#type: String,
    pub commodity_guid: String,
    pub commodity_scu: i64,
    pub non_std_scu: bool,
    pub parent_guid: String,
    pub code: String,
    pub description: String,
    pub hidden: bool,
    pub placeholder: bool,
}

impl<Q> Account<Q>
where
    Q: Query,
{
    pub(crate) fn from_with_query<T: AccountT>(item: &T, query: Arc<Q>) -> Self {
        Self {
            query,

            guid: item.guid(),
            name: item.name(),
            r#type: item.account_type(),
            commodity_guid: item.commodity_guid(),
            commodity_scu: item.commodity_scu(),
            non_std_scu: item.non_std_scu(),
            parent_guid: item.parent_guid(),
            code: item.code(),
            description: item.description(),
            hidden: item.hidden(),
            placeholder: item.placeholder(),
        }
    }

    pub async fn splits(&self) -> Result<Vec<Split<Q>>, Error> {
        let splits = SplitQ::account_guid(&*self.query, &self.guid).await?;
        Ok(splits
            .into_iter()
            .map(|x| Split::from_with_query(&x, self.query.clone()))
            .collect())
    }

    pub async fn parent(&self) -> Result<Option<Account<Q>>, Error> {
        if self.parent_guid.is_empty() {
            return Ok(None);
        };

        let mut accounts = AccountQ::guid(&*self.query, &self.parent_guid).await?;

        match accounts.pop() {
            None => Ok(None),
            Some(x) if accounts.is_empty() => {
                Ok(Some(Account::from_with_query(&x, self.query.clone())))
            }
            _ => Err(Error::GuidMultipleFound {
                model: "Account".to_string(),
                guid: self.parent_guid.clone(),
            }),
        }
    }

    pub async fn children(&self) -> Result<Vec<Account<Q>>, Error> {
        let accounts = AccountQ::parent_guid(&*self.query, &self.guid).await?;
        Ok(accounts
            .into_iter()
            .map(|x| Account::from_with_query(&x, self.query.clone()))
            .collect())
    }

    pub async fn commodity(&self) -> Result<Commodity<Q>, Error> {
        if self.commodity_guid.is_empty() {
            return Err(Error::GuidNotFound {
                model: "Commodity".to_string(),
                guid: self.commodity_guid.clone(),
            });
        }

        let mut commodities = CommodityQ::guid(&*self.query, &self.commodity_guid).await?;
        match commodities.pop() {
            None => Err(Error::GuidNotFound {
                model: "Commodity".to_string(),
                guid: self.commodity_guid.clone(),
            }),
            Some(x) if commodities.is_empty() => {
                Ok(Commodity::from_with_query(&x, self.query.clone()))
            }
            _ => Err(Error::GuidMultipleFound {
                model: "Commodity".to_string(),
                guid: self.commodity_guid.clone(),
            }),
        }
    }

    fn balance_into_currency<'a>(
        &'a self,
        currency: &'a Commodity<Q>,
        book: &'a Book<Q>,
    ) -> BoxFuture<'a, Result<crate::Num, Error>> {
        async {
            let mut net: crate::Num = self.splits().await?.iter().map(|s| s.quantity).sum();
            let commodity = self.commodity().await?;

            for child in self.children().await? {
                let child_net = child.balance_into_currency(&commodity, book).await?;
                net += child_net;
            }

            let rate = commodity.sell(currency, book).await.unwrap_or_else(|| {
                panic!(
                    "must have rate {} to {}",
                    commodity.mnemonic, currency.mnemonic
                )
            });
            // dbg!((
            //     &commodity.mnemonic,
            //     &currency.mnemonic,
            //     rate,
            //     &self.name,
            //     net
            // ));

            Ok(net * rate)
        }
        .boxed()
    }

    pub async fn balance(&self, book: &Book<Q>) -> Result<crate::Num, Error> {
        let mut net: crate::Num = self.splits().await?.iter().map(|s| s.quantity).sum();

        let commodity = self.commodity().await?;

        for child in self.children().await? {
            let child_net = child.balance_into_currency(&commodity, book).await?;
            net += child_net;
        }
        // dbg!((&self.name, net));

        Ok(net)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::sqlite::account::Account as AccountBase;
        use crate::SQLiteQuery;

        async fn setup() -> SQLiteQuery {
            let uri: &str = &format!(
                "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            println!("work_dir: {:?}", std::env::current_dir());
            SQLiteQuery::new(&format!("{uri}?mode=ro")).await.unwrap()
        }

        #[tokio::test]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = AccountBase {
                guid: "guid".to_string(),
                name: "name".to_string(),
                account_type: "account_type".to_string(),
                commodity_guid: Some("commodity_guid".to_string()),
                commodity_scu: 100,
                non_std_scu: 0,
                parent_guid: Some("parent_guid".to_string()),
                code: Some("code".to_string()),
                description: Some("description".to_string()),
                hidden: Some(0),
                placeholder: Some(1),
            };

            let result = Account::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.name, "name");
            assert_eq!(result.r#type, "account_type");
            assert_eq!(result.commodity_guid, "commodity_guid");
            assert_eq!(result.commodity_scu, 100);
            assert_eq!(result.non_std_scu, false);
            assert_eq!(result.parent_guid, "parent_guid");
            assert_eq!(result.code, "code");
            assert_eq!(result.description, "description");
            assert_eq!(result.hidden, false);
            assert_eq!(result.placeholder, true);
        }

        #[tokio::test]
        async fn test_splits() {
            let query = setup().await;
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
        async fn test_parent() {
            let query = setup().await;
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
        async fn test_no_parent() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Root Account")
                .await
                .unwrap()
                .unwrap();
            let parent = account.parent().await.unwrap();
            dbg!(&parent);
            assert!(parent.is_none());
        }

        #[tokio::test]
        async fn test_children() {
            let query = setup().await;
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
        async fn test_children2() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Asset")
                .await
                .unwrap()
                .unwrap();
            let children = account.children().await.unwrap();

            assert_eq!(children.len(), 3);
            assert_eq!(children[0].name, "Current");
            assert_eq!(children[1].name, "Fixed");
            assert_eq!(children[2].name, "Broker");
        }

        #[tokio::test]
        async fn test_commodity() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Cash")
                .await
                .unwrap()
                .unwrap();
            let commodity = account.commodity().await.unwrap();
            assert_eq!(commodity.mnemonic, "EUR");
        }

        #[tokio::test]
        async fn test_balance_into_currency() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Asset")
                .await
                .unwrap()
                .unwrap();
            let commodity = account.commodity().await.unwrap();

            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(
                f64,
                account
                    .balance_into_currency(&commodity, &book)
                    .await
                    .unwrap(),
                24695.3
            );
            #[cfg(feature = "decimal")]
            assert_eq!(
                account
                    .balance_into_currency(&commodity, &book)
                    .await
                    .unwrap(),
                Decimal::new(246_953, 1)
            );
        }

        #[tokio::test]
        async fn test_balance() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Current")
                .await
                .unwrap()
                .unwrap();

            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, account.balance(&book).await.unwrap(), 4590.0);
            #[cfg(feature = "decimal")]
            assert_eq!(account.balance(&book).await.unwrap(), Decimal::new(4590, 0));
        }
    }

    #[cfg(feature = "mysql")]
    mod mysql {
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::mysql::account::Account as AccountBase;
        use crate::MySQLQuery;

        async fn setup() -> MySQLQuery {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            MySQLQuery::new(uri).await.unwrap()
        }

        #[tokio::test]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = AccountBase {
                guid: "guid".to_string(),
                name: "name".to_string(),
                account_type: "account_type".to_string(),
                commodity_guid: Some("commodity_guid".to_string()),
                commodity_scu: 100,
                non_std_scu: 0,
                parent_guid: Some("parent_guid".to_string()),
                code: Some("code".to_string()),
                description: Some("description".to_string()),
                hidden: Some(0),
                placeholder: Some(1),
            };

            let result = Account::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.name, "name");
            assert_eq!(result.r#type, "account_type");
            assert_eq!(result.commodity_guid, "commodity_guid");
            assert_eq!(result.commodity_scu, 100);
            assert_eq!(result.non_std_scu, false);
            assert_eq!(result.parent_guid, "parent_guid");
            assert_eq!(result.code, "code");
            assert_eq!(result.description, "description");
            assert_eq!(result.hidden, false);
            assert_eq!(result.placeholder, true);
        }

        #[tokio::test]
        async fn test_splits() {
            let query = setup().await;
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
        async fn test_parent() {
            let query = setup().await;
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
        async fn test_no_parent() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Root Account")
                .await
                .unwrap()
                .unwrap();
            let parent = account.parent().await.unwrap();
            dbg!(&parent);
            assert!(parent.is_none());
        }

        #[tokio::test]
        async fn test_children() {
            let query = setup().await;
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
        async fn test_children2() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Asset")
                .await
                .unwrap()
                .unwrap();
            let children = account.children().await.unwrap();

            assert_eq!(children.len(), 3);
            assert_eq!(children[0].name, "Current");
            assert_eq!(children[1].name, "Fixed");
            assert_eq!(children[2].name, "Broker");
        }

        #[tokio::test]
        async fn test_commodity() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Cash")
                .await
                .unwrap()
                .unwrap();
            let commodity = account.commodity().await.unwrap();
            assert_eq!(commodity.mnemonic, "EUR");
        }

        #[tokio::test]
        async fn test_balance_into_currency() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Asset")
                .await
                .unwrap()
                .unwrap();
            let commodity = account.commodity().await.unwrap();

            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(
                f64,
                account
                    .balance_into_currency(&commodity, &book)
                    .await
                    .unwrap(),
                24695.3
            );
            #[cfg(feature = "decimal")]
            assert_eq!(
                account
                    .balance_into_currency(&commodity, &book)
                    .await
                    .unwrap(),
                Decimal::new(246_953, 1)
            );
        }

        #[tokio::test]
        async fn test_balance() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Current")
                .await
                .unwrap()
                .unwrap();

            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, account.balance(&book).await.unwrap(), 4590.0);
            #[cfg(feature = "decimal")]
            assert_eq!(account.balance(&book).await.unwrap(), Decimal::new(4590, 0));
        }
    }

    #[cfg(feature = "postgresql")]
    mod postgresql {
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::postgresql::account::Account as AccountBase;
        use crate::PostgreSQLQuery;

        async fn setup() -> PostgreSQLQuery {
            let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            PostgreSQLQuery::new(uri).await.unwrap()
        }

        #[tokio::test]
        async fn test_from_with_query() {
            let query = Arc::new(setup().await);
            let item = AccountBase {
                guid: "guid".to_string(),
                name: "name".to_string(),
                account_type: "account_type".to_string(),
                commodity_guid: Some("commodity_guid".to_string()),
                commodity_scu: 100,
                non_std_scu: 0,
                parent_guid: Some("parent_guid".to_string()),
                code: Some("code".to_string()),
                description: Some("description".to_string()),
                hidden: Some(0),
                placeholder: Some(1),
            };

            let result = Account::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.name, "name");
            assert_eq!(result.r#type, "account_type");
            assert_eq!(result.commodity_guid, "commodity_guid");
            assert_eq!(result.commodity_scu, 100);
            assert_eq!(result.non_std_scu, false);
            assert_eq!(result.parent_guid, "parent_guid");
            assert_eq!(result.code, "code");
            assert_eq!(result.description, "description");
            assert_eq!(result.hidden, false);
            assert_eq!(result.placeholder, true);
        }

        #[tokio::test]
        async fn test_splits() {
            let query = setup().await;
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
        async fn test_parent() {
            let query = setup().await;
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
        async fn test_no_parent() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Root Account")
                .await
                .unwrap()
                .unwrap();
            let parent = account.parent().await.unwrap();
            dbg!(&parent);
            assert!(parent.is_none());
        }

        #[tokio::test]
        async fn test_children() {
            let query = setup().await;
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
        async fn test_children2() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Asset")
                .await
                .unwrap()
                .unwrap();
            let children = account.children().await.unwrap();

            assert_eq!(children.len(), 3);
            assert_eq!(children[0].name, "Current");
            assert_eq!(children[1].name, "Fixed");
            assert_eq!(children[2].name, "Broker");
        }

        #[tokio::test]
        async fn test_commodity() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Cash")
                .await
                .unwrap()
                .unwrap();
            let commodity = account.commodity().await.unwrap();
            assert_eq!(commodity.mnemonic, "EUR");
        }

        #[tokio::test]
        async fn test_balance_into_currency() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Asset")
                .await
                .unwrap()
                .unwrap();
            let commodity = account.commodity().await.unwrap();

            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(
                f64,
                account
                    .balance_into_currency(&commodity, &book)
                    .await
                    .unwrap(),
                24695.3
            );
            #[cfg(feature = "decimal")]
            assert_eq!(
                account
                    .balance_into_currency(&commodity, &book)
                    .await
                    .unwrap(),
                Decimal::new(246_953, 1)
            );
        }

        #[tokio::test]
        async fn test_balance() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Current")
                .await
                .unwrap()
                .unwrap();

            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, account.balance(&book).await.unwrap(), 4590.0);
            #[cfg(feature = "decimal")]
            assert_eq!(account.balance(&book).await.unwrap(), Decimal::new(4590, 0));
        }
    }

    #[cfg(feature = "xml")]
    mod xml {
        use super::*;

        use pretty_assertions::assert_eq;

        use crate::query::xml::account::Account as AccountBase;
        use crate::XMLQuery;

        fn setup() -> XMLQuery {
            let path: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            println!("work_dir: {:?}", std::env::current_dir());
            XMLQuery::new(path).unwrap()
        }

        #[tokio::test]
        async fn test_from_with_query() {
            let query = Arc::new(setup());
            let item = AccountBase {
                guid: "guid".to_string(),
                name: "name".to_string(),
                account_type: "account_type".to_string(),
                commodity_guid: Some("commodity_guid".to_string()),
                commodity_scu: 100,
                non_std_scu: 0,
                parent_guid: Some("parent_guid".to_string()),
                code: Some("code".to_string()),
                description: Some("description".to_string()),
                hidden: false,
                placeholder: true,
            };

            let result = Account::from_with_query(&item, query);

            assert_eq!(result.guid, "guid");
            assert_eq!(result.name, "name");
            assert_eq!(result.r#type, "account_type");
            assert_eq!(result.commodity_guid, "commodity_guid");
            assert_eq!(result.commodity_scu, 100);
            assert_eq!(result.non_std_scu, false);
            assert_eq!(result.parent_guid, "parent_guid");
            assert_eq!(result.code, "code");
            assert_eq!(result.description, "description");
            assert_eq!(result.hidden, false);
            assert_eq!(result.placeholder, true);
        }

        #[tokio::test]
        async fn test_splits() {
            let query = setup();
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
        async fn test_parent() {
            let query = setup();
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
        async fn test_no_parent() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Root Account")
                .await
                .unwrap()
                .unwrap();
            let parent = account.parent().await.unwrap();
            dbg!(&parent);
            assert!(parent.is_none());
        }

        #[tokio::test]
        async fn test_children() {
            let query = setup();
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
        async fn test_children2() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Asset")
                .await
                .unwrap()
                .unwrap();
            let children = account.children().await.unwrap();

            assert_eq!(children.len(), 3);
            assert_eq!(children[0].name, "Current");
            assert_eq!(children[1].name, "Fixed");
            assert_eq!(children[2].name, "Broker");
        }

        #[tokio::test]
        async fn test_commodity() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Cash")
                .await
                .unwrap()
                .unwrap();
            let commodity = account.commodity().await.unwrap();
            assert_eq!(commodity.mnemonic, "EUR");
        }

        #[tokio::test]
        async fn test_balance_into_currency() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Asset")
                .await
                .unwrap()
                .unwrap();
            let commodity = account.commodity().await.unwrap();

            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(
                f64,
                account
                    .balance_into_currency(&commodity, &book)
                    .await
                    .unwrap(),
                24695.3
            );
            #[cfg(feature = "decimal")]
            assert_eq!(
                account
                    .balance_into_currency(&commodity, &book)
                    .await
                    .unwrap(),
                Decimal::new(246_953, 1)
            );
        }

        #[tokio::test]
        async fn test_balance() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Current")
                .await
                .unwrap()
                .unwrap();

            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, account.balance(&book).await.unwrap(), 4590.0);
            #[cfg(feature = "decimal")]
            assert_eq!(account.balance(&book).await.unwrap(), Decimal::new(4590, 0));
        }
    }
}
