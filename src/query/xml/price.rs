// ref: https://wiki.gnucash.org/wiki/GnuCash_XML_format

use chrono::NaiveDateTime;
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;
use xmltree::Element;

use crate::error::Error;
use crate::query::xml::XMLQuery;
use crate::query::{PriceQ, PriceT};

#[derive(Clone, Debug)]
pub struct Price {
    pub guid: String,
    pub commodity_guid: String,
    pub currency_guid: String,
    pub date: NaiveDateTime,
    pub source: Option<String>,
    pub r#type: Option<String>,
    pub value_num: i64,
    pub value_denom: i64,
}

impl TryFrom<&Element> for Price {
    type Error = Error;
    fn try_from(e: &Element) -> Result<Self, Error> {
        let guid = e
            .get_child("id")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned)
            .ok_or(Error::XMLFromElement {
                model: "Price guid".to_string(),
            })?;
        let commodity_guid = e
            .get_child("commodity")
            .and_then(|x| x.get_child("id"))
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned)
            .ok_or(Error::XMLFromElement {
                model: "Price commodity_guid".to_string(),
            })?;
        let currency_guid = e
            .get_child("currency")
            .and_then(|x| x.get_child("id"))
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned)
            .ok_or(Error::XMLFromElement {
                model: "Price currency_guid".to_string(),
            })?;
        let date = e
            .get_child("time")
            .and_then(|x| x.get_child("date"))
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned)
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z")
                    .expect("%Y-%m-%d %H:%M:%S%z")
            })
            .expect("time must exist");
        let source = e
            .get_child("source")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned);

        let r#type = e
            .get_child("type")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned);

        let splits = e
            .get_child("value")
            .expect("value must exist")
            .get_text()
            .unwrap();
        let mut splits = splits.split('/');
        let value_num = splits.next().unwrap().parse().unwrap();
        let value_denom = splits.next().unwrap().parse().unwrap();

        Ok(Self {
            guid,
            commodity_guid,
            currency_guid,
            date,
            source,
            r#type,
            value_num,
            value_denom,
        })
    }
}

impl PriceT for Price {
    fn guid(&self) -> String {
        self.guid.clone()
    }
    fn commodity_guid(&self) -> String {
        self.commodity_guid.clone()
    }
    fn currency_guid(&self) -> String {
        self.currency_guid.clone()
    }
    fn datetime(&self) -> NaiveDateTime {
        self.date
    }
    fn source(&self) -> String {
        self.source.clone().unwrap_or_default()
    }
    fn r#type(&self) -> String {
        self.r#type.clone().unwrap_or_default()
    }

    #[cfg(not(feature = "decimal"))]
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    fn value(&self) -> f64 {
        self.value_num as f64 / self.value_denom as f64
    }

    #[cfg(feature = "decimal")]
    #[must_use]
    fn value(&self) -> Decimal {
        Decimal::new(self.value_num, 0) / Decimal::new(self.value_denom, 0)
    }
}

impl PriceQ for XMLQuery {
    type P = Price;

    async fn all(&self) -> Result<Vec<Self::P>, Error> {
        let prices = match self.tree.get_child("pricedb") {
            None => Vec::new(),
            Some(node) => node
                .children
                .iter()
                .filter_map(xmltree::XMLNode::as_element)
                .filter(|e| e.name == "price")
                .map(Self::P::try_from)
                .collect::<Result<Vec<Self::P>, Error>>()?,
        };
        Ok(prices)
    }
    async fn guid(&self, guid: &str) -> Result<Vec<Self::P>, Error> {
        let results = self.all().await?;
        Ok(results.into_iter().filter(|x| x.guid == guid).collect())
    }
    async fn commodity_guid(&self, guid: &str) -> Result<Vec<Self::P>, Error> {
        let results = self.all().await?;
        Ok(results
            .into_iter()
            .filter(|x| x.commodity_guid == guid)
            .collect())
    }
    async fn currency_guid(&self, guid: &str) -> Result<Vec<Self::P>, Error> {
        let results = self.all().await?;
        Ok(results
            .into_iter()
            .filter(|x| x.currency_guid == guid)
            .collect())
    }
    async fn commodity_or_currency_guid(&self, guid: &str) -> Result<Vec<Self::P>, Error> {
        let results = self.all().await?;
        Ok(results
            .into_iter()
            .filter(|x| x.commodity_guid == guid || x.currency_guid == guid)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
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
            <price>
                <price:id type="guid">0d6684f44fb018e882de76094ed9c433</price:id>
                <price:commodity>
                    <cmdty:space>CURRENCY</cmdty:space>
                    <cmdty:id>ADF</cmdty:id>
                </price:commodity>
                <price:currency>
                    <cmdty:space>CURRENCY</cmdty:space>
                    <cmdty:id>AED</cmdty:id>
                </price:currency>
                <price:time>
                    <ts:date>2018-02-20 23:00:00 +0000</ts:date>
                </price:time>
                <price:source>user:price-editor</price:source>
                <price:type>unknown</price:type>
                <price:value>3/2</price:value>
            </price>
            </gnc-v2>
            "#;

        let e = Element::parse(data.as_bytes())
            .unwrap()
            .take_child("price")
            .unwrap();

        let price = Price::try_from(&e).unwrap();

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
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, 1.5, price.value());
        #[cfg(feature = "decimal")]
        assert_eq!(Decimal::new(15, 1), price.value());
    }

    #[tokio::test]
    async fn test_price() {
        let query = setup().await;
        let result = query
            .guid("0d6684f44fb018e882de76094ed9c433")
            .await
            .unwrap();

        let result = &result[0];
        assert_eq!(result.guid(), "0d6684f44fb018e882de76094ed9c433");
        assert_eq!(result.commodity_guid(), "ADF");
        assert_eq!(result.currency_guid(), "AED");
        assert_eq!(
            result.datetime(),
            NaiveDateTime::parse_from_str("2018-02-20 23:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(result.source(), "user:price-editor");
        assert_eq!(result.r#type(), "unknown");
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result.value(), Decimal::new(15, 1));
    }

    #[tokio::test]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_guid() {
        let query = setup().await;
        let result = query
            .guid("0d6684f44fb018e882de76094ed9c433")
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
        let result = query.commodity_guid("ADF").await.unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result[0].value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result[0].value(), Decimal::new(15, 1));
    }

    #[tokio::test]
    async fn currency_guid() {
        let query = setup().await;
        let result = query.currency_guid("AED").await.unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result[0].value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result[0].value(), Decimal::new(15, 1));
    }

    #[tokio::test]
    async fn commodity_or_currency_guid() {
        let query = setup().await;
        let result = query.commodity_or_currency_guid("AED").await.unwrap();
        assert_eq!(result.len(), 4);
    }
}
