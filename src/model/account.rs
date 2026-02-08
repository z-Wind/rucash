use std::sync::Arc;

use tracing::instrument;

use crate::Book;
use crate::error::Error;
use crate::model::{Commodity, Split};
use crate::query::{AccountQ, AccountT, CommodityQ, Query, SplitQ};

#[derive(Clone, Debug)]
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

    #[instrument(skip(self), fields(account_guid = %self.guid, account_name = %self.name))]
    pub async fn splits(&self) -> Result<Vec<Split<Q>>, Error> {
        tracing::debug!("fetching splits for account");
        let splits = SplitQ::account_guid(&*self.query, &self.guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch splits: {e}"))?;
        let result: Vec<_> = splits
            .into_iter()
            .map(|x| Split::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::debug!(count = result.len(), "splits fetched for account");
        Ok(result)
    }

    #[instrument(skip(self), fields(account_guid = %self.guid, parent_guid = %self.parent_guid))]
    pub async fn parent(&self) -> Result<Option<Account<Q>>, Error> {
        if self.parent_guid.is_empty() {
            tracing::debug!("no parent guid, returning None");
            return Ok(None);
        }

        tracing::debug!("fetching parent account");
        let mut accounts = AccountQ::guid(&*self.query, &self.parent_guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch parent account: {e}"))?;

        match accounts.pop() {
            None => {
                tracing::warn!("parent account not found");
                Ok(None)
            }
            Some(x) if accounts.is_empty() => {
                tracing::debug!("parent account found");
                Ok(Some(Account::from_with_query(&x, self.query.clone())))
            }
            _ => {
                tracing::error!("multiple parent accounts found for guid");
                Err(Error::GuidMultipleFound {
                    model: "Account".to_string(),
                    guid: self.parent_guid.clone(),
                })
            }
        }
    }

    #[instrument(skip(self), fields(account_guid = %self.guid))]
    pub async fn children(&self) -> Result<Vec<Account<Q>>, Error> {
        tracing::debug!("fetching children accounts");
        let accounts = AccountQ::parent_guid(&*self.query, &self.guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch children accounts: {e}"))?;
        let result: Vec<_> = accounts
            .into_iter()
            .map(|x| Account::from_with_query(&x, self.query.clone()))
            .collect();
        tracing::debug!(count = result.len(), "children accounts fetched");
        Ok(result)
    }

    #[instrument(skip(self), fields(commodity_guid = %self.commodity_guid))]
    pub async fn commodity(&self) -> Result<Commodity<Q>, Error> {
        if self.commodity_guid.is_empty() {
            tracing::error!("commodity guid is empty");
            return Err(Error::GuidNotFound {
                model: "Commodity".to_string(),
                guid: self.commodity_guid.clone(),
            });
        }

        tracing::debug!("fetching commodity for account");
        let mut commodities = CommodityQ::guid(&*self.query, &self.commodity_guid)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch commodity: {e}"))?;
        match commodities.pop() {
            None => {
                tracing::error!("commodity not found");
                Err(Error::GuidNotFound {
                    model: "Commodity".to_string(),
                    guid: self.commodity_guid.clone(),
                })
            }
            Some(x) if commodities.is_empty() => {
                tracing::debug!("commodity found for account");
                Ok(Commodity::from_with_query(&x, self.query.clone()))
            }
            _ => Err(Error::GuidMultipleFound {
                model: "Commodity".to_string(),
                guid: self.commodity_guid.clone(),
            }),
        }
    }

    #[instrument(skip(self, currency, book), fields(
        account_guid = %self.guid,
        account_name = %self.name,
        currency_mnemonic = %currency.mnemonic
    ))]
    async fn balance_into_currency<'a>(
        &'a self,
        currency: &'a Commodity<Q>,
        book: &'a Book<Q>,
    ) -> Result<crate::Num, Error> {
        tracing::debug!("calculating balance into currency");
        let splits = self
            .splits()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch splits: {e}"))?;
        let mut net: crate::Num = splits.iter().map(|s| s.quantity).sum();
        tracing::debug!(
            ?net,
            split_count = splits.len(),
            "calculated net from splits"
        );

        let commodity = self
            .commodity()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch commodity: {e}"))?;

        let children = self
            .children()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch children: {e}"))?;
        tracing::debug!(
            children_count = children.len(),
            "processing children balances"
        );

        for child in children {
            let child_net = Box::pin(child.balance_into_currency(&commodity, book))
                .await
                .inspect_err(|e| tracing::error!(child_account = %child.name, "failed to calculate child balance: {e}"))?;
            net += child_net;
        }

        let rate = commodity.sell(currency, book).await.unwrap_or_else(|| {
            tracing::error!(
                commodity = %commodity.mnemonic,
                currency = %currency.mnemonic,
                "no exchange rate available"
            );
            panic!(
                "must have rate {} to {}",
                commodity.mnemonic, currency.mnemonic
            )
        });

        let result = net * rate;
        tracing::debug!(?result, ?rate, "balance calculated in currency");
        Ok(result)
    }

    #[instrument(skip(self, book), fields(account_guid = %self.guid, account_name = %self.name))]
    pub async fn balance(&self, book: &Book<Q>) -> Result<crate::Num, Error> {
        tracing::debug!("calculating account balance");
        let splits = self
            .splits()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch splits: {e}"))?;
        let mut net: crate::Num = splits.iter().map(|s| s.quantity).sum();
        tracing::debug!(
            ?net,
            split_count = splits.len(),
            "calculated net from splits"
        );

        let commodity = self
            .commodity()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch commodity: {e}"))?;

        let children = self
            .children()
            .await
            .inspect_err(|e| tracing::error!("failed to fetch children: {e}"))?;
        tracing::debug!(
            children_count = children.len(),
            "processing children balances"
        );

        for child in children {
            let child_net = child.balance_into_currency(&commodity, book)
                .await
                .inspect_err(|e| tracing::error!(child_account = %child.name, "failed to calculate child balance: {e}"))?;
            net += child_net;
        }

        tracing::debug!(?net, "account balance calculated");
        Ok(net)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    use super::*;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::SQLiteQuery;
        use crate::query::sqlite::account::Account as AccountBase;

        use super::*;

        #[allow(clippy::unused_async)]
        async fn setup() -> SQLiteQuery {
            let uri: &str = &format!(
                "{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            tracing::info!("work_dir: {:?}", std::env::current_dir());
            SQLiteQuery::new(uri).unwrap()
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
        async fn test_no_parent() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Root Account")
                .await
                .unwrap()
                .unwrap();
            let parent = account.parent().await.unwrap();
            tracing::debug!(?parent);
            assert!(parent.is_none());
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
            assert!(
                children
                    .iter()
                    .map(|x| &x.name)
                    .any(|name| name == "Current"),
                "children does not contains Current"
            );
            assert!(
                children.iter().map(|x| &x.name).any(|name| name == "Fixed"),
                "children does not contains Fixed"
            );
            assert!(
                children
                    .iter()
                    .map(|x| &x.name)
                    .any(|name| name == "Broker"),
                "children does not contains Broker"
            );
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::MySQLQuery;
        use crate::query::mysql::account::Account as AccountBase;

        use super::*;

        async fn setup() -> MySQLQuery {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            MySQLQuery::new(uri).await.unwrap()
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
        async fn test_no_parent() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Root Account")
                .await
                .unwrap()
                .unwrap();
            let parent = account.parent().await.unwrap();
            tracing::debug!(?parent);
            assert!(parent.is_none());
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
            assert!(
                children
                    .iter()
                    .map(|x| &x.name)
                    .any(|name| name == "Current"),
                "children does not contains Current"
            );
            assert!(
                children.iter().map(|x| &x.name).any(|name| name == "Fixed"),
                "children does not contains Fixed"
            );
            assert!(
                children
                    .iter()
                    .map(|x| &x.name)
                    .any(|name| name == "Broker"),
                "children does not contains Broker"
            );
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::PostgreSQLQuery;
        use crate::query::postgresql::account::Account as AccountBase;

        use super::*;

        async fn setup() -> PostgreSQLQuery {
            let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            PostgreSQLQuery::new(uri).await.unwrap()
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
        async fn test_no_parent() {
            let query = setup().await;
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Root Account")
                .await
                .unwrap()
                .unwrap();
            let parent = account.parent().await.unwrap();
            tracing::debug!(?parent);
            assert!(parent.is_none());
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
            assert!(
                children
                    .iter()
                    .map(|x| &x.name)
                    .any(|name| name == "Current"),
                "children does not contains Current"
            );
            assert!(
                children.iter().map(|x| &x.name).any(|name| name == "Fixed"),
                "children does not contains Fixed"
            );
            assert!(
                children
                    .iter()
                    .map(|x| &x.name)
                    .any(|name| name == "Broker"),
                "children does not contains Broker"
            );
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::XMLQuery;
        use crate::query::xml::account::Account as AccountBase;

        use super::*;

        fn setup() -> XMLQuery {
            let path: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            tracing::info!("work_dir: {:?}", std::env::current_dir());
            XMLQuery::new(path).unwrap()
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
        async fn test_no_parent() {
            let query = setup();
            let book = Book::new(query).await.unwrap();
            let account = book
                .account_contains_name_ignore_case("Root Account")
                .await
                .unwrap()
                .unwrap();
            let parent = account.parent().await.unwrap();
            tracing::debug!(?parent);
            assert!(parent.is_none());
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
            assert!(
                children
                    .iter()
                    .map(|x| &x.name)
                    .any(|name| name == "Current"),
                "children does not contains Current"
            );
            assert!(
                children.iter().map(|x| &x.name).any(|name| name == "Fixed"),
                "children does not contains Fixed"
            );
            assert!(
                children
                    .iter()
                    .map(|x| &x.name)
                    .any(|name| name == "Broker"),
                "children does not contains Broker"
            );
        }

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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

        #[test(tokio::test)]
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
