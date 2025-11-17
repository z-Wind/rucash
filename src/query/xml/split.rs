// ref: https://wiki.gnucash.org/wiki/GnuCash_XML_format

use chrono::{DateTime, NaiveDateTime};
use roxmltree::Node;
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;

use super::XMLQuery;
use crate::error::Error;
use crate::query::{SplitQ, SplitT};

#[derive(Default, Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Split {
    pub guid: String,
    pub tx_guid: String,
    pub account_guid: String,
    pub memo: String,
    pub action: String,
    pub reconcile_state: bool,
    pub reconcile_date: Option<NaiveDateTime>,
    pub value_num: i64,
    pub value_denom: i64,
    pub quantity_num: i64,
    pub quantity_denom: i64,
    pub lot_guid: Option<String>,
}

impl XMLQuery {
    fn split_map(&self) -> Result<Arc<HashMap<String, Split>>, Error> {
        self.update_cache()?;
        Ok(self.splits.lock().unwrap().clone())
    }

    fn account_splits_map(&self) -> Result<Arc<HashMap<String, Vec<Split>>>, Error> {
        self.update_cache()?;
        Ok(self.account_splits.lock().unwrap().clone())
    }
}

impl Split {
    pub(super) fn try_from(tx_guid: String, n: Node) -> Result<Self, Error> {
        let mut split = Self {
            tx_guid,
            ..Self::default()
        };

        for child in n.children() {
            match child.tag_name().name() {
                "id" => {
                    split.guid = child
                        .text()
                        .ok_or(Error::XMLFromElement {
                            model: "Split guid".to_string(),
                        })?
                        .to_string();
                }
                "account" => {
                    split.account_guid = child
                        .text()
                        .ok_or(Error::XMLFromElement {
                            model: "Split account_guid".to_string(),
                        })?
                        .to_string();
                }
                "memo" => {
                    split.memo = child
                        .text()
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default();
                }
                "action" => {
                    split.action = child
                        .text()
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default();
                }
                "reconciled-state" => {
                    split.reconcile_state = child.text().is_some_and(|x| x != "n");
                }
                "reconcile-date" => {
                    split.reconcile_date = child
                        .first_element_child()
                        .and_then(|ts| ts.text())
                        .map(|x| chrono::NaiveDateTime::parse_from_str(x, "%Y-%m-%d %H:%M:%S %z"))
                        .transpose()?;
                }
                "value" => {
                    let mut splits =
                        child
                            .text()
                            .map(|s| s.split('/'))
                            .ok_or(Error::XMLFromElement {
                                model: "Split value".to_string(),
                            })?;
                    split.value_num = splits
                        .next()
                        .ok_or(Error::XMLFromElement {
                            model: "Split value value_num".to_string(),
                        })?
                        .parse()?;
                    split.value_denom = splits
                        .next()
                        .ok_or(Error::XMLFromElement {
                            model: "Split value value_denom".to_string(),
                        })?
                        .parse()?;
                }
                "quantity" => {
                    let mut splits =
                        child
                            .text()
                            .map(|s| s.split('/'))
                            .ok_or(Error::XMLFromElement {
                                model: "Split quantity".to_string(),
                            })?;
                    split.quantity_num = splits
                        .next()
                        .ok_or(Error::XMLFromElement {
                            model: "Split quantity quantity_num".to_string(),
                        })?
                        .parse()?;
                    split.quantity_denom = splits
                        .next()
                        .ok_or(Error::XMLFromElement {
                            model: "Split quantity quantity_denom".to_string(),
                        })?
                        .parse()?;
                }
                _ => {}
            }
        }

        Ok(split)
    }
}

impl SplitT for Split {
    fn guid(&self) -> String {
        self.guid.clone()
    }
    fn tx_guid(&self) -> String {
        self.tx_guid.clone()
    }
    fn account_guid(&self) -> String {
        self.account_guid.clone()
    }
    fn memo(&self) -> String {
        self.memo.clone()
    }
    fn action(&self) -> String {
        self.action.clone()
    }
    fn reconcile_state(&self) -> bool {
        self.reconcile_state
    }
    fn reconcile_datetime(&self) -> Option<NaiveDateTime> {
        let datetime = self.reconcile_date?;
        if datetime.and_utc() == DateTime::UNIX_EPOCH {
            return None;
        }
        Some(datetime)
    }
    fn lot_guid(&self) -> String {
        self.lot_guid.clone().unwrap_or_default()
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

    #[cfg(not(feature = "decimal"))]
    #[allow(clippy::cast_precision_loss)]
    fn quantity(&self) -> f64 {
        self.quantity_num as f64 / self.quantity_denom as f64
    }

    #[cfg(feature = "decimal")]
    fn quantity(&self) -> Decimal {
        Decimal::new(self.quantity_num, 0) / Decimal::new(self.quantity_denom, 0)
    }
}

impl SplitQ for XMLQuery {
    type S = Split;

    async fn all(&self) -> Result<Vec<Self::S>, Error> {
        let map = self.split_map()?;

        Ok(map.values().cloned().collect())
    }

    async fn guid(&self, guid: &str) -> Result<Vec<Self::S>, Error> {
        let map = self.split_map()?;

        Ok(map.get(guid).into_iter().cloned().collect())
    }

    async fn account_guid(&self, guid: &str) -> Result<Vec<Self::S>, Error> {
        let map = self.account_splits_map()?;

        Ok(map.get(guid).cloned().unwrap_or_default())
    }

    async fn tx_guid(&self, guid: &str) -> Result<Vec<Self::S>, Error> {
        let map = self.split_map()?;

        Ok(map
            .values()
            .filter(|x| x.tx_guid == guid)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
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
                    <trn:split>
                        <split:id type="guid">de832fe97e37811a7fff7e28b3a43425</split:id>
                        <split:reconciled-state>y</split:reconciled-state>
                        <split:reconcile-date>
                            <ts:date>2013-01-23 15:59:59 +0000</ts:date>
                        </split:reconcile-date>
                        <split:value>15000/100</split:value>
                        <split:quantity>15000/100</split:quantity>
                        <split:account type="guid">93fc043c3062aaa1297b30e543d2cd0d</split:account>
                    </trn:split>
                </gnc-v2>
                "#;

        let doc = Document::parse(data).unwrap();
        let n = doc.descendants().find(|n| n.has_tag_name("split")).unwrap();

        let split = Split::try_from(String::from("6c8876003c4a6026e38e3afb67d6f2b1"), n).unwrap();

        assert_eq!(split.guid, "de832fe97e37811a7fff7e28b3a43425");
        assert_eq!(split.tx_guid, "6c8876003c4a6026e38e3afb67d6f2b1");
        assert_eq!(split.account_guid, "93fc043c3062aaa1297b30e543d2cd0d");
        assert_eq!(split.memo, "");
        assert_eq!(split.action, "");
        assert_eq!(split.reconcile_state, true);
        assert_eq!(
            split.reconcile_date,
            Some("2013-01-23T15:59:59".parse().unwrap(),)
        );
        assert_eq!(split.value_num, 15000);
        assert_eq!(split.value_denom, 100);
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, 150.0, split.value());
        #[cfg(feature = "decimal")]
        assert_eq!(Decimal::new(150, 0), split.value());
        assert_eq!(split.quantity_num, 15000);
        assert_eq!(split.quantity_denom, 100);
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, 150.0, split.quantity());
        #[cfg(feature = "decimal")]
        assert_eq!(Decimal::new(150, 0), split.quantity());
        assert_eq!(split.lot_guid, None);
    }

    #[tokio::test]
    async fn test_split() {
        let query = setup().await;
        let result = query
            .guid("de832fe97e37811a7fff7e28b3a43425")
            .await
            .unwrap();

        let result = &result[0];
        assert_eq!(result.guid(), "de832fe97e37811a7fff7e28b3a43425");
        assert_eq!(result.tx_guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
        assert_eq!(result.account_guid(), "93fc043c3062aaa1297b30e543d2cd0d");
        assert_eq!(result.memo(), "");
        assert_eq!(result.action(), "");
        assert_eq!(result.reconcile_state(), false);
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

    #[tokio::test]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 25);
    }

    #[tokio::test]
    async fn test_guid() {
        let query = setup().await;
        let result = query
            .guid("de832fe97e37811a7fff7e28b3a43425")
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
        let result = query
            .account_guid("93fc043c3062aaa1297b30e543d2cd0d")
            .await
            .unwrap();
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_tx_guid() {
        let query = setup().await;
        let result = query
            .tx_guid("6c8876003c4a6026e38e3afb67d6f2b1")
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
    }
}
