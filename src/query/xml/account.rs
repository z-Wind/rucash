// ref: https://wiki.gnucash.org/wiki/GnuCash_XML_format

use roxmltree::Node;
use std::sync::Arc;
use tracing::instrument;

use super::XMLQuery;
use crate::error::Error;
use crate::query::{AccountQ, AccountT};

#[allow(clippy::struct_field_names)]
#[derive(Default, Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Account {
    pub(crate) guid: String,
    pub(crate) name: String,
    pub(crate) account_type: String,
    pub(crate) commodity_guid: Option<String>,
    pub(crate) commodity_scu: i64,
    pub(crate) non_std_scu: i64,
    pub(crate) parent_guid: Option<String>,
    pub(crate) code: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) hidden: bool,
    pub(crate) placeholder: bool,
}

impl XMLQuery {
    fn accounts_map(&self) -> Result<super::AccountMap, Error> {
        self.update_cache()?;

        let cache = self
            .cache
            .read()
            .map_err(|e| Error::Internal(format!("Cache lock poisoned: {e}")))?;

        Ok(Arc::clone(&cache.accounts))
    }

    fn commodity_accounts_map(&self) -> Result<super::AccountsMap, Error> {
        self.update_cache()?;

        let cache = self
            .cache
            .read()
            .map_err(|e| Error::Internal(format!("Cache lock poisoned: {e}")))?;

        Ok(Arc::clone(&cache.commodity_accounts))
    }

    fn same_parent_accounts_map(&self) -> Result<super::AccountsMap, Error> {
        self.update_cache()?;

        let cache = self
            .cache
            .read()
            .map_err(|e| Error::Internal(format!("Cache lock poisoned: {e}")))?;

        Ok(Arc::clone(&cache.same_parent_accounts))
    }

    fn name_accounts_map(&self) -> Result<super::AccountsMap, Error> {
        self.update_cache()?;

        let cache = self
            .cache
            .read()
            .map_err(|e| Error::Internal(format!("Cache lock poisoned: {e}")))?;

        Ok(Arc::clone(&cache.name_accounts))
    }
}

impl TryFrom<Node<'_, '_>> for Account {
    type Error = Error;
    fn try_from(n: Node) -> Result<Self, Error> {
        let mut account = Self::default();
        for child in n.children() {
            match child.tag_name().name() {
                "name" => {
                    account.name = child
                        .text()
                        .ok_or_else(|| Error::XMLMissingField {
                            model: "Account".to_string(),
                            field: "name".to_string(),
                        })?
                        .to_string();
                }
                "id" => {
                    account.guid = child
                        .text()
                        .ok_or_else(|| Error::XMLMissingField {
                            model: "Account".to_string(),
                            field: "guid".to_string(),
                        })?
                        .to_string();
                }
                "type" => {
                    account.account_type = child
                        .text()
                        .ok_or_else(|| Error::XMLMissingField {
                            model: "Account".to_string(),
                            field: "type".to_string(),
                        })?
                        .to_string();
                }
                "commodity" => {
                    account.commodity_guid = child
                        .children()
                        .find(|n| n.has_tag_name("id"))
                        .and_then(|n| n.text())
                        .map(std::string::ToString::to_string);
                }
                "commodity-scu" => {
                    account.commodity_scu = child.text().map_or(Ok(0), str::parse::<i64>)?;
                }
                "non-std-scu" => {
                    account.non_std_scu = child.text().map_or(Ok(0), str::parse::<i64>)?;
                }
                "parent" => {
                    account.parent_guid = child.text().map(std::string::ToString::to_string);
                }
                "code" => {
                    account.code = child.text().map(std::string::ToString::to_string);
                }
                "description" => {
                    account.description = child.text().map(std::string::ToString::to_string);
                }
                "hidden" => {
                    account.hidden = child.text().is_some_and(|x| x == "true");
                }
                "slots" => {
                    account.placeholder = child
                        .descendants()
                        .find(|n| n.has_tag_name("key"))
                        .and_then(|n| n.next_sibling_element())
                        .and_then(|n| n.text())
                        .is_some_and(|x| x == "true");
                }
                _ => {}
            }
        }

        Ok(account)
    }
}

impl AccountT for Account {
    fn guid(&self) -> String {
        self.guid.clone()
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn account_type(&self) -> String {
        self.account_type.clone()
    }
    fn commodity_guid(&self) -> String {
        self.commodity_guid.clone().unwrap_or_default()
    }
    fn commodity_scu(&self) -> i64 {
        self.commodity_scu
    }
    fn non_std_scu(&self) -> bool {
        self.non_std_scu != 0
    }
    fn parent_guid(&self) -> String {
        self.parent_guid.clone().unwrap_or_default()
    }
    fn code(&self) -> String {
        self.code.clone().unwrap_or_default()
    }
    fn description(&self) -> String {
        self.description.clone().unwrap_or_default()
    }
    fn hidden(&self) -> bool {
        self.hidden
    }
    fn placeholder(&self) -> bool {
        self.placeholder
    }
}

impl AccountQ for XMLQuery {
    type Item = Account;

    #[instrument(skip(self))]
    async fn all(&self) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching all accounts from xml");
        let map = self
            .accounts_map()
            .inspect_err(|e| tracing::error!("failed to get account map: {e}"))?;

        let result: Vec<_> = map.values().map(|x| (**x).clone()).collect();
        tracing::debug!(count = result.len(), "accounts fetched from xml");
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn guid(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching account by guid from xml");
        let map = self
            .accounts_map()
            .inspect_err(|e| tracing::error!("failed to get account map: {e}"))?;

        let result: Vec<_> = map.get(guid).into_iter().map(|x| (**x).clone()).collect();
        tracing::debug!(count = result.len(), "accounts found by guid");
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn commodity_guid(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching accounts by commodity_guid from xml");
        let map = self
            .commodity_accounts_map()
            .inspect_err(|e| tracing::error!("failed to get commodity accounts map: {e}"))?;

        let result: Vec<_> = map
            .get(guid)
            .map(|v| v.iter().map(|x| (**x).clone()).collect())
            .unwrap_or_default();
        tracing::debug!(count = result.len(), "accounts found by commodity_guid");
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn parent_guid(&self, guid: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching accounts by parent_guid from xml");
        let map = self
            .same_parent_accounts_map()
            .inspect_err(|e| tracing::error!("failed to get parent accounts map: {e}"))?;

        let result: Vec<_> = map
            .get(guid)
            .map(|v| v.iter().map(|x| (**x).clone()).collect())
            .unwrap_or_default();
        tracing::debug!(count = result.len(), "accounts found by parent_guid");
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn name(&self, name: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("fetching accounts by name from xml");
        let map = self
            .name_accounts_map()
            .inspect_err(|e| tracing::error!("failed to get name accounts map: {e}"))?;

        let result: Vec<_> = map
            .get(name)
            .map(|v| v.iter().map(|x| (**x).clone()).collect())
            .unwrap_or_default();
        tracing::debug!(count = result.len(), "accounts found by name");
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn contains_name_ignore_case(&self, name: &str) -> Result<Vec<Self::Item>, Error> {
        tracing::debug!("searching accounts with name pattern from xml");
        let map = self
            .accounts_map()
            .inspect_err(|e| tracing::error!("failed to get account map: {e}"))?;

        let result: Vec<_> = map
            .values()
            .filter(|x| x.name.to_lowercase().contains(&name.to_lowercase()))
            .map(|x| (**x).clone())
            .collect();
        tracing::debug!(count = result.len(), "accounts found matching name pattern");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use roxmltree::Document;
    use test_log::test;
    use tokio::sync::OnceCell;

    use super::*;

    static Q: OnceCell<XMLQuery> = OnceCell::const_new();
    async fn setup() -> &'static XMLQuery {
        Q.get_or_init(|| async {
            let path: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            tracing::debug!("work_dir: {:?}", std::env::current_dir());
            XMLQuery::new(path).unwrap()
        })
        .await
    }

    #[test]
    fn test_try_from_element() {
        let data = r#"<?xml version="1.0" encoding="utf-8" ?>
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
            "#;

        let doc = Document::parse(data).unwrap();
        let n = doc
            .descendants()
            .find(|n| n.has_tag_name("account"))
            .unwrap();

        let account = Account::try_from(n).unwrap();

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
        assert_eq!(account.hidden, false);
        assert_eq!(account.placeholder, true);
    }

    #[test(tokio::test)]
    async fn test_account() {
        let query = setup().await;
        let result = query
            .guid("fcd795021c976ba75621ec39e75f6214")
            .await
            .unwrap();

        let result = &result[0];
        assert_eq!(result.guid(), "fcd795021c976ba75621ec39e75f6214");
        assert_eq!(result.name(), "Asset");
        assert_eq!(result.account_type(), "ASSET");
        assert_eq!(result.commodity_guid(), "EUR");
        assert_eq!(result.commodity_scu(), 100);
        assert_eq!(result.non_std_scu(), false);
        assert_eq!(result.parent_guid(), "00622dda21937b29e494179de5013f82");
        assert_eq!(result.code(), "");
        assert_eq!(result.description(), "");
        assert_eq!(result.hidden(), false);
        assert_eq!(result.placeholder(), true);
    }

    #[test(tokio::test)]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 20);
    }

    #[test(tokio::test)]
    async fn test_guid() {
        let query = setup().await;
        let result = query
            .guid("fcd795021c976ba75621ec39e75f6214")
            .await
            .unwrap();

        assert_eq!(result[0].name, "Asset");
    }

    #[test(tokio::test)]
    async fn test_commodity_guid() {
        let query = setup().await;
        let result = query.commodity_guid("EUR").await.unwrap();
        assert_eq!(result.len(), 14);
    }

    #[test(tokio::test)]
    async fn test_parent_guid() {
        let query = setup().await;
        let result = query
            .parent_guid("fcd795021c976ba75621ec39e75f6214")
            .await
            .unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test(tokio::test)]
    async fn test_name() {
        let query = setup().await;
        let result = query.name("Asset").await.unwrap();
        assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
    }

    #[test(tokio::test)]
    async fn test_contains_name_ignore_case() {
        let query = setup().await;
        let result = query.contains_name_ignore_case("AS").await.unwrap();
        assert_eq!(result.len(), 3);
    }
}
