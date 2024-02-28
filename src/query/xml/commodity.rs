// ref: https://wiki.gnucash.org/wiki/GnuCash_XML_format

use itertools::Itertools;
use xmltree::Element;

use crate::error::Error;
use crate::query::xml::XMLQuery;
use crate::query::{CommodityQ, CommodityT};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
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

impl TryFrom<&Element> for Commodity {
    type Error = Error;
    fn try_from(e: &Element) -> Result<Self, Error> {
        let guid = e
            .get_child("id")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned)
            .ok_or(Error::XMLFromElement {
                model: "Commodity guid".to_string(),
            })?;
        let namespace = e
            .get_child("space")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned)
            .ok_or(Error::XMLFromElement {
                model: "Commodity namespace".to_string(),
            })?;
        let mnemonic = guid.clone();
        let fullname = e
            .get_child("name")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned);
        let cusip = e
            .get_child("cusip")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned);
        let fraction = e
            .get_child("fraction")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned)
            .map_or(100, |x| x.parse().expect("must be i32"));
        let quote_flag = e.get_child("get_quotes").is_some();
        let quote_source = e
            .get_child("quote_source")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned);
        let quote_tz = e
            .get_child("quote_tz")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned);

        Ok(Self {
            guid,
            namespace,
            mnemonic,
            fullname,
            cusip,
            fraction,
            quote_flag,
            quote_source,
            quote_tz,
        })
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
        let mut commodities: Vec<Self::C> = self
            .tree
            .children
            .iter()
            .filter_map(xmltree::XMLNode::as_element)
            .filter(|e| e.name == "commodity")
            .map(Self::C::try_from)
            .collect::<Result<Vec<Self::C>, Error>>()?;

        let mut prices: Vec<Self::C> = match self.tree.get_child("pricedb") {
            None => Vec::new(),
            Some(node) => node
                .children
                .iter()
                .filter_map(xmltree::XMLNode::as_element)
                .filter(|e| e.name == "price")
                .flat_map(|e| {
                    let cmdty = e.get_child("commodity").ok_or(Error::XMLFromElement {
                        model: "Commodity price commodity".to_string(),
                    });
                    let cmdty = cmdty.map(Self::C::try_from);

                    let crncy = e.get_child("currency").ok_or(Error::XMLFromElement {
                        model: "Commodity price currency".to_string(),
                    });
                    let crncy = crncy.map(Self::C::try_from);

                    vec![cmdty, crncy]
                })
                .collect::<Result<Result<Vec<Self::C>, Error>, Error>>()??,
        };
        commodities.append(&mut prices);

        commodities.sort_unstable_by(|c1, c2| c1.guid.cmp(&c2.guid));
        Ok(commodities
            .into_iter()
            .dedup_by(|x, y| x.guid == y.guid)
            .collect())
    }

    async fn guid(&self, guid: &str) -> Result<Vec<Self::C>, Error> {
        let results = self.all().await?;
        Ok(results.into_iter().filter(|x| x.guid == guid).collect())
    }

    async fn namespace(&self, namespace: &str) -> Result<Vec<Self::C>, Error> {
        let results = self.all().await?;
        Ok(results
            .into_iter()
            .filter(|x| x.namespace == namespace)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
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

        let e = Element::parse(data.as_bytes())
            .unwrap()
            .take_child("commodity")
            .unwrap();

        let commodity = Commodity::try_from(&e).unwrap();

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
