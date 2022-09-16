use super::model;
use flate2::read::GzDecoder;
use itertools::Itertools;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use xmltree::Element;

use std::ops::Deref;

#[derive(Debug)]
pub struct DataWithPool<T> {
    content: T,
    pub pool: XMLPool,
}

impl<T> DataWithPool<T> {
    pub fn new(content: T, pool: XMLPool) -> Self
    where
        T: model::NullNone,
    {
        Self {
            content: content.null_none(),
            pool,
        }
    }

    pub fn content(&self) -> &T {
        &self.content
    }
}

impl<T> Deref for DataWithPool<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.content
    }
}

impl<T> PartialEq for DataWithPool<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

impl<T> PartialOrd for DataWithPool<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.content.partial_cmp(&other.content)
    }
}

#[derive(Debug)]
pub struct XMLPool(Arc<Element>);

impl Clone for XMLPool {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl XMLPool {
    /// read gnucash xml file in gzip
    fn new(uri: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let f = File::open(uri)?;
        let mut d = GzDecoder::new(f);
        let mut data = String::new();
        d.read_to_string(&mut data)?;

        let mut root: Element = Element::parse(data.as_bytes())?;
        root = root.take_child("book").ok_or("None")?;

        Ok(Self(Arc::new(root)))
    }

    fn accounts(&self) -> Vec<DataWithPool<model::Account>> {
        self.0
            .children
            .iter()
            .filter_map(|x| x.as_element())
            .filter(|e| e.name == "account")
            .map(|e| {
                let content = model::Account::new_by_element(e);
                DataWithPool::<model::Account>::new(content, self.clone())
            })
            .collect()
    }

    fn splits(&self) -> Vec<DataWithPool<model::Split>> {
        self.0
            .children
            .iter()
            .filter_map(|x| x.as_element())
            .filter(|e| e.name == "transaction")
            .flat_map(|e| {
                let tx_guid = e
                    .get_child("id")
                    .expect("id")
                    .get_text()
                    .expect("text")
                    .into_owned();

                e.get_child("splits")
                    .expect("splits")
                    .children
                    .iter()
                    .filter_map(|e| e.as_element())
                    .map(move |e| {
                        let content = model::Split::new_by_element(tx_guid.clone(), e);
                        DataWithPool::<model::Split>::new(content, self.clone())
                    })
            })
            .collect()
    }

    fn transactions(&self) -> Vec<DataWithPool<model::Transaction>> {
        self.0
            .children
            .iter()
            .filter_map(|x| x.as_element())
            .filter(|e| e.name == "transaction")
            .map(|e| {
                let content = model::Transaction::new_by_element(e);
                DataWithPool::<model::Transaction>::new(content, self.clone())
            })
            .collect()
    }

    fn prices(&self) -> Vec<DataWithPool<model::Price>> {
        match self.0.get_child("pricedb") {
            None => Vec::new(),
            Some(node) => node
                .children
                .iter()
                .filter_map(|x| x.as_element())
                .filter(|e| e.name == "price")
                .map(|e| {
                    let content = model::Price::new_by_element(e);
                    DataWithPool::<model::Price>::new(content, self.clone())
                })
                .collect(),
        }
    }

    fn commodities(&self) -> Vec<DataWithPool<model::Commodity>> {
        let mut commodities: Vec<DataWithPool<model::Commodity>> = self
            .0
            .children
            .iter()
            .filter_map(|x| x.as_element())
            .filter(|e| e.name == "commodity")
            .map(|e| {
                let content = model::Commodity::new_by_element(e);
                DataWithPool::<model::Commodity>::new(content, self.clone())
            })
            .collect();

        let mut prices: Vec<DataWithPool<model::Commodity>> = match self.0.get_child("pricedb") {
            None => Vec::new(),
            Some(node) => node
                .children
                .iter()
                .filter_map(|x| x.as_element())
                .filter(|e| e.name == "price")
                .flat_map(|e| {
                    let cmdty = e.get_child("commodity").expect("must exist");
                    let content = model::Commodity::new_by_element(cmdty);
                    let cmdty = DataWithPool::<model::Commodity>::new(content, self.clone());

                    let crncy = e.get_child("currency").expect("must exist");
                    let content = model::Commodity::new_by_element(crncy);
                    let crncy = DataWithPool::<model::Commodity>::new(content, self.clone());

                    vec![cmdty, crncy]
                })
                .collect(),
        };
        commodities.append(&mut prices);

        commodities.sort_unstable_by(|c1, c2| c1.guid.cmp(&c2.guid));
        commodities
            .into_iter()
            .dedup_by(|x, y| x.guid == y.guid)
            .collect()
    }
}

#[derive(Debug)]
pub struct XMLBook {
    pool: Arc<XMLPool>,
}

impl XMLBook {
    /// read gnucash xml file in gzip
    pub fn new(uri: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = Arc::new(XMLPool::new(uri)?);
        Ok(Self { pool })
    }

    pub fn accounts(&self) -> Vec<DataWithPool<model::Account>> {
        self.pool.accounts()
    }

    pub fn account_by_name(&self, name: &str) -> Option<DataWithPool<model::Account>> {
        self.accounts_contains_name(name).pop()
    }

    pub fn accounts_contains_name(&self, name: &str) -> Vec<DataWithPool<model::Account>> {
        self.accounts()
            .into_iter()
            .filter(|x| x.name.to_lowercase().contains(&name.to_lowercase()))
            .collect()
    }

    pub fn splits(&self) -> Vec<DataWithPool<model::Split>> {
        self.pool.splits()
    }

    pub fn transactions(&self) -> Vec<DataWithPool<model::Transaction>> {
        self.pool.transactions()
    }

    pub fn prices(&self) -> Vec<DataWithPool<model::Price>> {
        self.pool.prices()
    }

    pub fn commodities(&self) -> Vec<DataWithPool<model::Commodity>> {
        self.pool.commodities()
    }

    pub fn currencies(&self) -> Vec<DataWithPool<model::Commodity>> {
        self.commodities()
            .into_iter()
            .filter(|x| x.namespace == "CURRENCY")
            .collect()
    }
}

impl model::Account {
    fn new_by_element(e: &Element) -> Self {
        let guid = e
            .get_child("id")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("id must exist");
        let name = e
            .get_child("name")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("name must exist");
        let account_type = e
            .get_child("type")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("type must exist");
        let commodity_guid = e
            .get_child("commodity")
            .and_then(|x| x.get_child("id"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let commodity_scu = e
            .get_child("commodity-scu")
            .and_then(|x| x.get_text())
            .map(|x| x.parse().expect("must be i32"))
            .unwrap_or(0);
        let non_std_scu = e
            .get_child("non-std-scu")
            .and_then(|x| x.get_text())
            .map(|x| x.parse().expect("must be i32"))
            .unwrap_or(0);
        let parent_guid = e
            .get_child("parent")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let code = e
            .get_child("code")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let description = e
            .get_child("description")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let hidden = e
            .get_child("hidden")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| x.parse().unwrap_or(0));

        let slots: Vec<&Element> = match e.get_child("slots") {
            None => Vec::new(),
            Some(x) => x.children.iter().filter_map(|x| x.as_element()).collect(),
        };
        let placeholder = slots
            .iter()
            .find(|e| {
                e.get_child("key")
                    .and_then(|e| e.get_text())
                    .map(|s| s.into_owned())
                    == Some("placeholder".to_string())
            })
            .and_then(|e| e.get_child("value"))
            .and_then(|s| s.get_text())
            .map(|x| x.into_owned())
            .map(|x| if x == "true" { 1 } else { 0 });

        model::Account {
            guid,
            name,
            account_type,
            commodity_guid,
            commodity_scu,
            non_std_scu,
            parent_guid,
            code,
            description,
            hidden,
            placeholder,
        }
    }
}

impl DataWithPool<model::Account> {
    pub fn splits(&self) -> Vec<DataWithPool<model::Split>> {
        self.pool
            .splits()
            .into_iter()
            .filter(|x| x.account_guid == self.guid)
            .collect()
    }
    pub fn parent(&self) -> Option<DataWithPool<model::Account>> {
        self.pool
            .accounts()
            .into_iter()
            .find(|x| Some(x.guid.clone()) == self.parent_guid)
    }
    pub fn children(&self) -> Vec<DataWithPool<model::Account>> {
        self.pool
            .accounts()
            .into_iter()
            .filter(|x| x.parent_guid == Some(self.guid.clone()))
            .collect()
    }
    pub fn commodity(&self) -> Option<DataWithPool<model::Commodity>> {
        self.pool
            .commodities()
            .into_iter()
            .find(|x| Some(x.guid.clone()) == self.commodity_guid)
    }

    pub fn balance(&self) -> f64 {
        let splits = self.splits();
        let net:f64 = splits.iter().map(|x| x.quantity).sum::<f64>()
        + self.children().iter().map(|child| child.balance()).sum::<f64>();

        let commodity = match self.commodity() {
            Some(commodity) => commodity,
            None => return net,
        };

        let currency = match self.parent().and_then(|p| p.commodity()) {
            Some(currency) => currency,
            None => return net,
        };

        commodity
            .sell(&currency)
            .map(|rate| net * rate)
            .unwrap_or(net)
    }
}

impl model::Split {
    fn new_by_element(tx_guid: String, e: &Element) -> Self {
        let guid = e
            .get_child("id")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("id must exist");
        let tx_guid = tx_guid;
        let account_guid = e
            .get_child("account")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("account must exist");
        let memo = e
            .get_child("memo")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .unwrap_or(String::from(""));
        let action = e
            .get_child("action")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .unwrap_or(String::from(""));
        let reconcile_state = e
            .get_child("reconciled-state")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .unwrap_or(String::from(""));
        let reconcile_date = e
            .get_child("reconciled-date")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S")
                    .expect("%Y-%m-%d %H:%M:%S")
            });

        let splits = e
            .get_child("value")
            .expect("value must exist")
            .get_text()
            .unwrap();
        let mut splits = splits.split('/');
        let value_num = splits.next().unwrap().parse().unwrap();
        let value_denom = splits.next().unwrap().parse().unwrap();
        let value = value_num as f64 / value_denom as f64;

        let splits = e
            .get_child("quantity")
            .expect("quantity must exist")
            .get_text()
            .unwrap();
        let mut splits = splits.split('/');
        let quantity_num = splits.next().unwrap().parse().unwrap();
        let quantity_denom = splits.next().unwrap().parse().unwrap();
        let quantity = quantity_num as f64 / quantity_denom as f64;
        let lot_guid = None;

        model::Split {
            guid,
            tx_guid,
            account_guid,
            memo,
            action,
            reconcile_state,
            reconcile_date,
            value_num,
            value_denom,
            value,
            quantity_num,
            quantity_denom,
            quantity,
            lot_guid,
        }
    }
}

impl DataWithPool<model::Split> {
    pub fn transaction(&self) -> DataWithPool<model::Transaction> {
        self.pool
            .transactions()
            .into_iter()
            .find(|x| x.guid == self.tx_guid)
            .expect("tx_guid must match one")
    }

    pub fn account(&self) -> DataWithPool<model::Account> {
        self.pool
            .accounts()
            .into_iter()
            .find(|x| x.guid == self.account_guid)
            .expect("tx_guid must match one")
    }
}

impl model::Transaction {
    fn new_by_element(e: &Element) -> Self {
        let guid = e
            .get_child("id")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("id must exist");
        let currency_guid = e
            .get_child("currency")
            .and_then(|x| x.get_child("id"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("currency must exist");
        let num = e
            .get_child("num")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .unwrap_or(String::from(""));
        let post_date = e
            .get_child("date-posted")
            .and_then(|x| x.get_child("date"))
            .and_then(|x| x.get_text())
            // .map(|x| x.into_owned())
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z")
                    .expect("date-posted must be %Y-%m-%d %H:%M:%S%z")
            });
        let enter_date = e
            .get_child("date-entered")
            .and_then(|x| x.get_child("date"))
            .and_then(|x| x.get_text())
            // .map(|x| x.into_owned())
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z")
                    .expect("date-entered must be %Y-%m-%d %H:%M:%S%z")
            });
        let description = e
            .get_child("description")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());

        model::Transaction {
            guid,
            currency_guid,
            num,
            post_date,
            enter_date,
            description,
        }
    }
}

impl DataWithPool<model::Transaction> {
    pub fn currency(&self) -> DataWithPool<model::Commodity> {
        self.pool
            .commodities()
            .into_iter()
            .find(|x| x.guid == self.currency_guid)
            .expect("tx_guid must match one")
    }

    pub fn splits(&self) -> Vec<DataWithPool<model::Split>> {
        self.pool
            .splits()
            .into_iter()
            .filter(|x| x.tx_guid == self.guid)
            .collect()
    }
}

impl model::Price {
    fn new_by_element(e: &Element) -> Self {
        let guid = e
            .get_child("id")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("id must exist");
        let commodity_guid = e
            .get_child("commodity")
            .and_then(|x| x.get_child("id"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("commodity must exist");
        let currency_guid = e
            .get_child("currency")
            .and_then(|x| x.get_child("id"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("currency must exist");
        let date = e
            .get_child("time")
            .and_then(|x| x.get_child("date"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z")
                    .expect("%Y-%m-%d %H:%M:%S%z")
            })
            .expect("time must exist");
        let source = e
            .get_child("source")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());

        let r#type = e
            .get_child("type")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());

        let splits = e
            .get_child("value")
            .expect("value must exist")
            .get_text()
            .unwrap();
        let mut splits = splits.split('/');
        let value_num = splits.next().unwrap().parse().unwrap();
        let value_denom = splits.next().unwrap().parse().unwrap();
        let value = value_num as f64 / value_denom as f64;

        model::Price {
            guid,
            commodity_guid,
            currency_guid,
            date,
            source,
            r#type,
            value_num,
            value_denom,
            value,
        }
    }
}

impl DataWithPool<model::Price> {
    pub fn commodity(&self) -> DataWithPool<model::Commodity> {
        self.pool
            .commodities()
            .into_iter()
            .find(|x| x.guid == self.commodity_guid)
            .expect("tx_guid must match one")
    }

    pub fn currency(&self) -> DataWithPool<model::Commodity> {
        self.pool
            .commodities()
            .into_iter()
            .find(|x| x.guid == self.currency_guid)
            .expect("tx_guid must match one")
    }
}

impl model::Commodity {
    fn new_by_element(e: &Element) -> Self {
        let guid = e
            .get_child("id")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("id must exist");
        let namespace = e
            .get_child("space")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .expect("space must exist");
        let mnemonic = guid.clone();
        let fullname = e
            .get_child("fullname")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let cusip = e
            .get_child("cusip")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let fraction = e
            .get_child("fraction")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| x.parse().expect("must be i32"))
            .unwrap_or(100);
        let quote_flag = match e.get_child("get_quotes") {
            Some(_) => 1,
            None => 0,
        };
        let quote_source = e
            .get_child("quote_source")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let quote_tz = e
            .get_child("quote_tz")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());

        model::Commodity {
            guid,
            namespace,
            mnemonic,
            fullname,
            cusip,
            fraction,
            quote_flag,
            quote_source,
            quote_tz,
        }
    }
}

impl DataWithPool<model::Commodity> {
    pub fn accounts(&self) -> Vec<DataWithPool<model::Account>> {
        self.pool
            .accounts()
            .into_iter()
            .filter(|x| x.commodity_guid == Some(self.guid.clone()))
            .collect()
    }

    pub fn transactions(&self) -> Vec<DataWithPool<model::Transaction>> {
        self.pool
            .transactions()
            .into_iter()
            .filter(|x| x.currency_guid == self.guid)
            .collect()
    }

    pub fn as_commodity_prices(&self) -> Vec<DataWithPool<model::Price>> {
        self.pool
            .prices()
            .into_iter()
            .filter(|x| x.commodity_guid == self.guid)
            .collect()
    }

    pub fn as_currency_prices(&self) -> Vec<DataWithPool<model::Price>> {
        self.pool
            .prices()
            .into_iter()
            .filter(|x| x.currency_guid == self.guid)
            .collect()
    }

    pub fn as_commodity_or_currency_prices(&self) -> Vec<DataWithPool<model::Price>> {
        self.pool
            .prices()
            .into_iter()
            .filter(|x| x.commodity_guid == self.guid || x.currency_guid == self.guid)
            .collect()
    }

    pub fn sell(&self, currency: &DataWithPool<model::Commodity>) -> Option<f64> {
        if self.guid == currency.guid {
            return Some(1.0);
        }

        let mut prices: Vec<DataWithPool<model::Price>> = self
            .as_commodity_or_currency_prices()
            .into_iter()
            .filter(|x| x.currency_guid == currency.guid || x.commodity_guid == currency.guid)
            .collect();
        prices.sort_by_key(|x| x.date);

        let p = prices.last()?;

        if self.guid == p.commodity_guid && currency.guid == p.currency_guid {
            Some(p.value)
        } else if self.guid == p.currency_guid && currency.guid == p.commodity_guid {
            Some(p.value_denom as f64 / p.value_num as f64)
        } else {
            None
        }
    }

    pub fn buy(&self, commodity: &DataWithPool<model::Commodity>) -> Option<f64> {
        commodity.sell(&self).map(|v| 1.0 / v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //     mod book {
    //         use super::*;
    //     }
    mod account {
        use super::*;
        #[test]
        fn new_by_element() {
            let data = r##"
            <?xml version="1.0" encoding="utf-8" ?>
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
            "##;

            let e = Element::parse(data.as_bytes())
                .unwrap()
                .take_child("account")
                .unwrap();

            let account = model::Account::new_by_element(&e);

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
            assert_eq!(account.hidden, None);
            assert_eq!(account.placeholder.unwrap(), 1);
        }
    }

    mod split {
        use super::*;
        #[test]
        fn new_by_element() {
            let data = r##"
            <?xml version="1.0" encoding="utf-8" ?>
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
                    <split:reconciled-state>n</split:reconciled-state>
                    <split:value>15000/100</split:value>
                    <split:quantity>15000/100</split:quantity>
                    <split:account type="guid">93fc043c3062aaa1297b30e543d2cd0d</split:account>
                </trn:split>
            </gnc-v2>
            "##;

            let e = Element::parse(data.as_bytes())
                .unwrap()
                .take_child("split")
                .unwrap();

            let split =
                model::Split::new_by_element(String::from("6c8876003c4a6026e38e3afb67d6f2b1"), &e);

            assert_eq!(split.guid, "de832fe97e37811a7fff7e28b3a43425");
            assert_eq!(split.tx_guid, "6c8876003c4a6026e38e3afb67d6f2b1");
            assert_eq!(split.account_guid, "93fc043c3062aaa1297b30e543d2cd0d");
            assert_eq!(split.memo, "");
            assert_eq!(split.action, "");
            assert_eq!(split.reconcile_state, "n");
            assert_eq!(split.reconcile_date, None);
            assert_eq!(split.value_num, 15000);
            assert_eq!(split.value_denom, 100);
            assert_eq!(split.value, 150.0);
            assert_eq!(split.quantity_num, 15000);
            assert_eq!(split.quantity_denom, 100);
            assert_eq!(split.quantity, 150.0);
            assert_eq!(split.lot_guid, None);
        }
    }

    mod transaction {
        use super::*;
        #[test]
        fn new_by_element() {
            let data = r##"
            <?xml version="1.0" encoding="utf-8" ?>
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
            "##;

            let e = Element::parse(data.as_bytes())
                .unwrap()
                .take_child("transaction")
                .unwrap();

            let transaction = model::Transaction::new_by_element(&e);

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
    }

    mod price {
        use super::*;
        #[test]
        fn new_by_element() {
            let data = r##"
            <?xml version="1.0" encoding="utf-8" ?>
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
            "##;

            let e = Element::parse(data.as_bytes())
                .unwrap()
                .take_child("price")
                .unwrap();

            let price = model::Price::new_by_element(&e);

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
            assert_eq!(price.value, 1.5);
        }
    }

    mod commodity {
        use super::*;
        #[test]
        fn new_by_element() {
            let data = r##"
            <?xml version="1.0" encoding="utf-8" ?>
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
            "##;

            let e = Element::parse(data.as_bytes())
                .unwrap()
                .take_child("commodity")
                .unwrap();

            let commodity = model::Commodity::new_by_element(&e);

            assert_eq!(commodity.guid, "EUR");
            assert_eq!(commodity.namespace, "CURRENCY");
            assert_eq!(commodity.mnemonic, "EUR");
            assert_eq!(commodity.fullname, None);
            assert_eq!(commodity.cusip, None);
            assert_eq!(commodity.fraction, 100);
            assert_eq!(commodity.quote_flag, 1);
            assert_eq!(commodity.quote_source.as_ref().unwrap(), "currency");
            assert_eq!(commodity.quote_tz, None);
        }
    }
}
