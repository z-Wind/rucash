// ref: https://wiki.gnucash.org/wiki/GnuCash_XML_format

use chrono::NaiveDateTime;
use roxmltree::{Document, Node};
use std::collections::HashMap;
use std::sync::Arc;

use super::{FileTimeIndex, XMLQuery};
use crate::error::Error;
use crate::query::{TransactionQ, TransactionT};

#[derive(Default, Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Transaction {
    pub guid: String,
    pub currency_guid: String,
    pub num: String,
    pub post_date: Option<NaiveDateTime>,
    pub enter_date: Option<NaiveDateTime>,
    pub description: Option<String>,
}

impl XMLQuery {
    fn transaction_map(&self) -> Result<Arc<HashMap<String, Transaction>>, Error> {
        let mut cache = self.transactions.lock().unwrap();
        if let Some(cache) = &*cache
            && self.is_file_unchanged(FileTimeIndex::Transactions as usize)?
        {
            return Ok(cache.clone());
        }

        let data = self.gnucash_data()?;
        let doc = Document::parse(&data)?;

        let transactions = doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("book"))
            .expect("must exist book")
            .children()
            .filter(|n| n.has_tag_name("transaction"))
            .map(|n| {
                let result = Transaction::try_from(n);
                result.map(|t| (t.guid.clone(), t))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let transactions = Arc::new(transactions);
        *cache = Some(transactions.clone());

        Ok(transactions)
    }
}

impl TryFrom<Node<'_, '_>> for Transaction {
    type Error = Error;

    fn try_from(n: Node) -> Result<Self, Error> {
        let mut transaction = Self::default();
        for child in n.children() {
            match child.tag_name().name() {
                "id" => {
                    transaction.guid = child
                        .text()
                        .ok_or(Error::XMLFromElement {
                            model: "Transaction guid".to_string(),
                        })?
                        .to_string();
                }
                "currency" => {
                    transaction.currency_guid = child
                        .children()
                        .find(|n| n.has_tag_name("id"))
                        .and_then(|n| n.text())
                        .map(std::string::ToString::to_string)
                        .ok_or(Error::XMLFromElement {
                            model: "Transaction currency_guid".to_string(),
                        })?;
                }
                "num" => {
                    transaction.num = child
                        .text()
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default();
                }
                "date-posted" => {
                    transaction.post_date = child
                        .children()
                        .find(|n| n.has_tag_name("date"))
                        .and_then(|n| n.text())
                        .map(|x| chrono::NaiveDateTime::parse_from_str(x, "%Y-%m-%d %H:%M:%S%z"))
                        .transpose()?;
                }
                "date-entered" => {
                    transaction.enter_date = child
                        .children()
                        .find(|n| n.has_tag_name("date"))
                        .and_then(|n| n.text())
                        .map(|x| chrono::NaiveDateTime::parse_from_str(x, "%Y-%m-%d %H:%M:%S%z"))
                        .transpose()?;
                }
                "description" => {
                    transaction.description = child.text().map(std::string::ToString::to_string);
                }

                _ => {}
            }
        }

        Ok(transaction)
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
        let map = self.transaction_map()?;

        Ok(map.values().cloned().collect())
    }

    async fn guid(&self, guid: &str) -> Result<Vec<Self::T>, Error> {
        let map = self.transaction_map()?;

        Ok(map.get(guid).into_iter().cloned().collect())
    }

    async fn currency_guid(&self, guid: &str) -> Result<Vec<Self::T>, Error> {
        let map = self.transaction_map()?;

        Ok(map
            .values()
            .filter(|x| x.currency_guid == guid)
            .cloned()
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

        let doc = Document::parse(data).unwrap();
        let n = doc
            .descendants()
            .find(|n| n.has_tag_name("transaction"))
            .unwrap();

        let transaction = Transaction::try_from(n).unwrap();

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
