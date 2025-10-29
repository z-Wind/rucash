// ref: https://wiki.gnucash.org/wiki/GnuCash_XML_format

use chrono::NaiveDateTime;
use roxmltree::{Document, Node};
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;

use crate::error::Error;
use crate::query::xml::XMLQuery;
use crate::query::{PriceQ, PriceT};

#[derive(Default, Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
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

impl TryFrom<Node<'_, '_>> for Price {
    type Error = Error;
    fn try_from(n: Node) -> Result<Self, Error> {
        let mut price = Self::default();
        for child in n.children() {
            match child.tag_name().name() {
                "id" => {
                    price.guid = child
                        .text()
                        .ok_or(Error::XMLFromElement {
                            model: "Price guid".to_string(),
                        })?
                        .to_string();
                }
                "commodity" => {
                    price.commodity_guid = child
                        .children()
                        .find(|n| n.has_tag_name("id"))
                        .and_then(|n| n.text())
                        .map(std::string::ToString::to_string)
                        .ok_or(Error::XMLFromElement {
                            model: "Price commodity_guid".to_string(),
                        })?;
                }
                "currency" => {
                    price.currency_guid = child
                        .children()
                        .find(|n| n.has_tag_name("id"))
                        .and_then(|n| n.text())
                        .map(std::string::ToString::to_string)
                        .ok_or(Error::XMLFromElement {
                            model: "Price currency_guid".to_string(),
                        })?;
                }
                "time" => {
                    price.date = child
                        .children()
                        .find(|n| n.has_tag_name("date"))
                        .and_then(|n| n.text())
                        .map(|x| chrono::NaiveDateTime::parse_from_str(x, "%Y-%m-%d %H:%M:%S%z"))
                        .ok_or(Error::XMLFromElement {
                            model: "Price time".to_string(),
                        })??;
                }
                "source" => {
                    price.source = child.text().map(std::string::ToString::to_string);
                }
                "type" => {
                    price.r#type = child.text().map(std::string::ToString::to_string);
                }
                "value" => {
                    let mut splits =
                        child
                            .text()
                            .map(|s| s.split('/'))
                            .ok_or(Error::XMLFromElement {
                                model: "Price value".to_string(),
                            })?;
                    price.value_num = splits
                        .next()
                        .ok_or(Error::XMLFromElement {
                            model: "Price value value_num".to_string(),
                        })?
                        .parse()?;
                    price.value_denom = splits
                        .next()
                        .ok_or(Error::XMLFromElement {
                            model: "Price value value_denom".to_string(),
                        })?
                        .parse()?;
                }
                _ => {}
            }
        }

        Ok(price)
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
    #[allow(clippy::cast_precision_loss)]
    fn value(&self) -> f64 {
        self.value_num as f64 / self.value_denom as f64
    }

    #[cfg(feature = "decimal")]
    fn value(&self) -> Decimal {
        Decimal::new(self.value_num, 0) / Decimal::new(self.value_denom, 0)
    }
}

impl PriceQ for XMLQuery {
    type P = Price;

    async fn all(&self) -> Result<Vec<Self::P>, Error> {
        let mut cache = self.prices.lock().unwrap();
        if let Some(cache) = cache.clone() {
            return Ok(cache);
        }

        let doc = Document::parse(&self.text)?;

        let prices = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .expect("must exist book")
            .children()
            .find(|n| n.has_tag_name("pricedb"))
            .map_or_else(
                || Ok(Vec::new()),
                |n| {
                    n.children()
                        .filter(|n| n.has_tag_name("price"))
                        .map(Self::P::try_from)
                        .collect()
                },
            )?;

        *cache = Some(prices.clone());

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

        let doc = Document::parse(data).unwrap();
        let n = doc.descendants().find(|n| n.has_tag_name("price")).unwrap();

        let price = Price::try_from(n).unwrap();

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
