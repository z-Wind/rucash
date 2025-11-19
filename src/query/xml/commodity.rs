// ref: https://wiki.gnucash.org/wiki/GnuCash_XML_format

use roxmltree::Node;

use super::XMLQuery;
use crate::error::Error;
use crate::query::{CommodityQ, CommodityT};

#[derive(Default, Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Commodity {
    pub(crate) guid: String,
    pub(crate) namespace: String,
    pub(crate) mnemonic: String,
    pub(crate) fullname: Option<String>,
    pub(crate) cusip: Option<String>,
    pub(crate) fraction: i64,
    pub(crate) quote_flag: bool,
    pub(crate) quote_source: Option<String>,
    pub(crate) quote_tz: Option<String>,
}

impl XMLQuery {
    fn commodity_map(&self) -> Result<super::CommodityMap, Error> {
        self.update_cache()?;
        Ok(self.commodities.lock().unwrap().clone())
    }

    fn namespace_commodities_map(&self) -> Result<super::CommoditiesMap, Error> {
        self.update_cache()?;
        Ok(self.namespace_commodities.lock().unwrap().clone())
    }
}

impl TryFrom<Node<'_, '_>> for Commodity {
    type Error = Error;
    fn try_from(n: Node) -> Result<Self, Error> {
        let mut commodity = Self {
            fraction: 100,
            ..Self::default()
        };

        for child in n.children() {
            match child.tag_name().name() {
                "id" => {
                    commodity.guid = child
                        .text()
                        .ok_or(Error::XMLFromElement {
                            model: "Commodity guid".to_string(),
                        })?
                        .to_string();
                    commodity.mnemonic = commodity.guid.clone();
                }
                "space" => {
                    commodity.namespace = child
                        .text()
                        .ok_or(Error::XMLFromElement {
                            model: "Commodity namespacee".to_string(),
                        })?
                        .to_string();
                }
                "name" => {
                    commodity.fullname = child.text().map(std::string::ToString::to_string);
                }
                "cusip" => {
                    commodity.cusip = child.text().map(std::string::ToString::to_string);
                }
                "fraction" => {
                    commodity.fraction = child
                        .text()
                        .map_or(100, |x| x.parse().expect("must be i32"));
                }
                "get_quotes" => {
                    commodity.quote_flag = true;
                }
                "quote_source" => {
                    commodity.quote_source = child.text().map(std::string::ToString::to_string);
                }
                "quote_tz" => {
                    commodity.quote_tz = child.text().map(std::string::ToString::to_string);
                }
                _ => {}
            }
        }

        Ok(commodity)
    }
}

impl CommodityT for Commodity {
    fn guid(&self) -> String {
        self.guid.clone()
    }
    fn namespace(&self) -> String {
        self.namespace.clone()
    }
    fn mnemonic(&self) -> String {
        self.mnemonic.clone()
    }
    // not support in xml
    fn fullname(&self) -> String {
        self.fullname.clone().unwrap_or_default()
    }
    // not support in xml
    fn cusip(&self) -> String {
        self.cusip.clone().unwrap_or_default()
    }
    fn fraction(&self) -> i64 {
        self.fraction
    }
    fn quote_flag(&self) -> bool {
        self.quote_flag
    }
    fn quote_source(&self) -> String {
        self.quote_source.clone().unwrap_or_default()
    }
    fn quote_tz(&self) -> String {
        self.quote_tz.clone().unwrap_or_default()
    }
}

impl CommodityQ for XMLQuery {
    type C = Commodity;

    async fn all(&self) -> Result<Vec<Self::C>, Error> {
        let map = self.commodity_map()?;

        Ok(map.values().map(|x| (**x).clone()).collect())
    }

    async fn guid(&self, guid: &str) -> Result<Vec<Self::C>, Error> {
        let map = self.commodity_map()?;

        Ok(map.get(guid).map(|x| (**x).clone()).into_iter().collect())
    }

    async fn namespace(&self, namespace: &str) -> Result<Vec<Self::C>, Error> {
        let map = self.namespace_commodities_map()?;

        Ok(map
            .get(namespace)
            .map(|v| v.iter().map(|x| (**x).clone()).collect())
            .unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use roxmltree::Document;
    use tokio::sync::OnceCell;

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
            <gnc:commodity version="2.0.0">
                <cmdty:space>CURRENCY</cmdty:space>
                <cmdty:id>EUR</cmdty:id>
                <cmdty:get_quotes/>
                <cmdty:quote_source>currency</cmdty:quote_source>
                <cmdty:quote_tz/>
            </gnc:commodity>
            </gnc-v2>
            "#;

        let doc = Document::parse(data).unwrap();
        let n = doc
            .descendants()
            .find(|n| n.has_tag_name("commodity"))
            .unwrap();

        let commodity = Commodity::try_from(n).unwrap();

        assert_eq!(commodity.guid, "EUR");
        assert_eq!(commodity.namespace, "CURRENCY");
        assert_eq!(commodity.mnemonic, "EUR");
        assert_eq!(commodity.fullname, None);
        assert_eq!(commodity.cusip, None);
        assert_eq!(commodity.fraction, 100);
        assert_eq!(commodity.quote_flag, true);
        assert_eq!(commodity.quote_source.as_ref().unwrap(), "currency");
        assert_eq!(commodity.quote_tz, None);
    }

    #[tokio::test]
    async fn test_commodity() {
        let query = setup().await;
        let result = query.guid("EUR").await.unwrap();

        let result = &result[0];
        assert_eq!(result.guid(), "EUR");
        assert_eq!(result.namespace(), "CURRENCY");
        assert_eq!(result.mnemonic(), "EUR");
        assert_eq!(result.fullname(), "");
        assert_eq!(result.cusip(), "");
        assert_eq!(result.fraction(), 100);
        assert_eq!(result.quote_flag(), true);
        assert_eq!(result.quote_source(), "currency");
        assert_eq!(result.quote_tz(), "");
    }

    #[tokio::test]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 6);
    }

    #[tokio::test]
    async fn test_guid() {
        let query = setup().await;
        let result = query.guid("EUR").await.unwrap();
        assert_eq!(result[0].guid, "EUR");
    }

    #[tokio::test]
    async fn test_namespace() {
        let query = setup().await;
        let result = query.namespace("CURRENCY").await.unwrap();
        assert_eq!(result.len(), 4);
    }
}
