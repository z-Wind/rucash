// ref: https://wiki.gnucash.org/wiki/GnuCash_XML_format

use chrono::NaiveDateTime;
use xmltree::Element;

use crate::error::Error;
use crate::query::xml::XMLQuery;
use crate::query::{TransactionQ, TransactionT};

#[derive(Clone, Debug)]
pub struct Transaction {
    pub guid: String,
    pub currency_guid: String,
    pub num: String,
    pub post_date: Option<NaiveDateTime>,
    pub enter_date: Option<NaiveDateTime>,
    pub description: Option<String>,
}

impl TryFrom<&Element> for Transaction {
    type Error = Error;
    fn try_from(e: &Element) -> Result<Self, Error> {
        let guid = e
            .get_child("id")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned)
            .ok_or(Error::XMLFromElement {
                model: "Transaction guid".to_string(),
            })?;
        let currency_guid = e
            .get_child("currency")
            .and_then(|x| x.get_child("id"))
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned)
            .ok_or(Error::XMLFromElement {
                model: "Transaction currency_guid".to_string(),
            })?;
        let num = e
            .get_child("num")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned)
            .unwrap_or_default();
        let post_date = e
            .get_child("date-posted")
            .and_then(|x| x.get_child("date"))
            .and_then(xmltree::Element::get_text)
            // .map(std::borrow::Cow::into_owned)
            .map(|x| chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z"))
            .transpose()?;
        let enter_date = e
            .get_child("date-entered")
            .and_then(|x| x.get_child("date"))
            .and_then(xmltree::Element::get_text)
            // .map(std::borrow::Cow::into_owned)
            .map(|x| chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z"))
            .transpose()?;
        let description = e
            .get_child("description")
            .and_then(xmltree::Element::get_text)
            .map(std::borrow::Cow::into_owned);

        Ok(Self {
            guid,
            currency_guid,
            num,
            post_date,
            enter_date,
            description,
        })
    }
}

impl TransactionT for Transaction {
    fn guid(&self) -> String {
        self.guid.clone()
    }
    fn currency_guid(&self) -> String {
        self.currency_guid.clone()
    }
    fn num(&self) -> String {
        self.num.clone()
    }
    fn post_datetime(&self) -> NaiveDateTime {
        self.post_date.expect("transaction post_date should exist")
    }
    fn enter_datetime(&self) -> NaiveDateTime {
        self.enter_date
            .expect("transaction enter_date should exist")
    }
    fn description(&self) -> String {
        self.description.clone().unwrap_or_default()
    }
}

impl TransactionQ for XMLQuery {
    type T = Transaction;

    async fn all(&self) -> Result<Vec<Self::T>, Error> {
        self.tree
            .children
            .iter()
            .filter_map(xmltree::XMLNode::as_element)
            .filter(|e| e.name == "transaction")
            .map(Self::T::try_from)
            .collect()
    }

    async fn guid(&self, guid: &str) -> Result<Vec<Self::T>, Error> {
        let results = self.all().await?;
        Ok(results.into_iter().filter(|x| x.guid == guid).collect())
    }

    async fn currency_guid(&self, guid: &str) -> Result<Vec<Self::T>, Error> {
        let results = self.all().await?;
        Ok(results
            .into_iter()
            .filter(|x| x.currency_guid == guid)
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
            <gnc:transaction version="2.0.0">
                <trn:id type="guid">6c8876003c4a6026e38e3afb67d6f2b1</trn:id>
                <trn:currency>
                    <cmdty:space>CURRENCY</cmdty:space>
                    <cmdty:id>EUR</cmdty:id>
                </trn:currency>
                <trn:date-posted>
                    <ts:date>2014-12-24 10:59:00 +0000</ts:date>
                </trn:date-posted>
                <trn:date-entered>
                    <ts:date>2014-12-25 10:08:15 +0000</ts:date>
                </trn:date-entered>
                <trn:description>income 1</trn:description>
                <trn:splits>
                    <trn:split>
                        <split:id type="guid">de832fe97e37811a7fff7e28b3a43425</split:id>
                        <split:reconciled-state>n</split:reconciled-state>
                        <split:value>15000/100</split:value>
                        <split:quantity>15000/100</split:quantity>
                        <split:account type="guid">93fc043c3062aaa1297b30e543d2cd0d</split:account>
                    </trn:split>
                    <trn:split>
                        <split:id type="guid">1e612f650eb598d9e803902b6aca73e3</split:id>
                        <split:reconciled-state>n</split:reconciled-state>
                        <split:value>-15000/100</split:value>
                        <split:quantity>-15000/100</split:quantity>
                        <split:account type="guid">6bbc8f20544452cac1637fb9a9b851bb</split:account>
                    </trn:split>
                </trn:splits>
            </gnc:transaction>
            </gnc-v2>
            "#;

        let e = Element::parse(data.as_bytes())
            .unwrap()
            .take_child("transaction")
            .unwrap();

        let transaction = Transaction::try_from(&e).unwrap();

        assert_eq!(transaction.guid, "6c8876003c4a6026e38e3afb67d6f2b1");
        assert_eq!(transaction.currency_guid, "EUR");
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
    async fn test_transaction() {
        let query = setup().await;
        let result = query
            .guid("6c8876003c4a6026e38e3afb67d6f2b1")
            .await
            .unwrap();

        let result = &result[0];
        assert_eq!(result.guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
        assert_eq!(result.currency_guid(), "EUR");
        assert_eq!(result.num(), "");
        assert_eq!(
            result.post_datetime(),
            NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(
            result.enter_datetime(),
            NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(result.description(), "income 1");
    }

    #[tokio::test]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 11);
    }

    #[tokio::test]
    async fn test_by_guid() {
        let query = setup().await;
        let result = query
            .guid("6c8876003c4a6026e38e3afb67d6f2b1")
            .await
            .unwrap();

        assert_eq!(
            result[0].post_date.unwrap(),
            NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );

        assert_eq!(
            result[0].enter_date.unwrap(),
            NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S").unwrap()
        );
    }

    #[tokio::test]
    async fn test_currency_guid() {
        let query = setup().await;
        let result = query.currency_guid("EUR").await.unwrap();

        assert_eq!(result.len(), 11);
    }
}
