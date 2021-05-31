use flate2::read::GzDecoder;
use itertools::Itertools;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use xmltree::Element;

use super::template::BookT;
use super::template::{AccountT, CommodityT, PriceT, SplitT, TransactionT};
use super::template::{Book, Item};
use super::template::{_Account, _Commodity, _Price, _Split, _Transaction};

pub type DB = Element;
type Error = Box<dyn std::error::Error>;

pub type Account = Item<_Account, DB>;
pub type Split = Item<_Split, DB>;
pub type Transaction = Item<_Transaction, DB>;
pub type Price = Item<_Price, DB>;
pub type Commodity = Item<_Commodity, DB>;
pub type XMLBook = Book<DB>;

impl XMLBook {
    fn _accounts(db: &Rc<DB>) -> Result<Vec<Account>, Error> {
        let result = db
            .children
            .iter()
            .filter(|x| x.as_element().unwrap().name == "account")
            .map(|x| {
                let e = x.as_element().unwrap();
                let content = _Account::new_by_element(e);
                Account::new(content, &db)
            })
            .collect();

        Ok(result)
    }

    fn _splits(db: &Rc<DB>) -> Result<Vec<Split>, Error> {
        let result = db
            .children
            .iter()
            .filter(|x| x.as_element().unwrap().name == "transaction")
            .flat_map(|x| {
                let e = x.as_element().unwrap();
                let tx_guid = e.get_child("id").unwrap().get_text().unwrap().into_owned();

                e.get_child("splits")
                    .unwrap()
                    .children
                    .iter()
                    .map(move |x| {
                        let e = x.as_element().unwrap();
                        let content = _Split::new_by_element(tx_guid.clone(), e);
                        Split::new(content, &db)
                    })
            })
            .collect();

        Ok(result)
    }

    fn _transactions(db: &Rc<DB>) -> Result<Vec<Transaction>, Error> {
        let result = db
            .children
            .iter()
            .filter(|x| x.as_element().unwrap().name == "transaction")
            .map(|x| {
                let e = x.as_element().unwrap();
                let content = _Transaction::new_by_element(e);
                Transaction::new(content, &db)
            })
            .collect();

        Ok(result)
    }

    fn _prices(db: &Rc<DB>) -> Result<Vec<Price>, Error> {
        let result = db
            .get_child("pricedb")
            .expect("pricedb")
            .children
            .iter()
            .filter(|x| x.as_element().unwrap().name == "price")
            .map(|x| {
                let e = x.as_element().unwrap();
                let content = _Price::new_by_element(e);
                Price::new(content, &db)
            })
            .collect();

        Ok(result)
    }

    fn _commodities(db: &Rc<DB>) -> Result<Vec<Commodity>, Error> {
        let commodity_chain = db
            .children
            .iter()
            .filter(|x| x.as_element().unwrap().name == "commodity")
            .map(|x| {
                let e = x.as_element().unwrap();
                let content = _Commodity::new_by_element(e);
                Commodity::new(content, &db)
            });

        let price_chain = db
            .get_child("pricedb")
            .expect("pricedb")
            .children
            .iter()
            .filter(|x| x.as_element().unwrap().name == "price")
            .flat_map(|x| {
                let e = x.as_element().unwrap();

                let cmdty = e.get_child("commodity").unwrap();
                let content = _Commodity::new_by_element(cmdty);
                let cmdty = Commodity::new(content, &db);

                let crncy = e.get_child("currency").unwrap();
                let content = _Commodity::new_by_element(crncy);
                let crncy = Commodity::new(content, &db);

                vec![cmdty, crncy]
            });

        let mut result = commodity_chain
            .chain(price_chain)
            .collect::<Vec<Commodity>>();
        result.sort_by(|d1, d2| d1.guid.cmp(&d2.guid));
        let result = result
            .into_iter()
            .dedup_by(|x, y| x.guid == y.guid)
            .collect();

        Ok(result)
    }
}

impl BookT for XMLBook {
    type DB = DB;

    /// read gnucash xml file in gzip
    fn new(uri: &str) -> Result<Self, Error> {
        let f = File::open(uri)?;
        let mut d = GzDecoder::new(f);
        let mut data = String::new();
        d.read_to_string(&mut data).unwrap();

        let mut root: Element = Element::parse(data.as_bytes()).unwrap();
        root = root.take_child("book").unwrap();

        let db = Rc::new(root);
        Ok(Book { db })
    }

    fn accounts(&self) -> Result<Vec<Account>, Error> {
        Book::_accounts(&self.db)
    }

    fn splits(&self) -> Result<Vec<Split>, Error> {
        Book::_splits(&self.db)
    }

    fn transactions(&self) -> Result<Vec<Transaction>, Error> {
        Book::_transactions(&self.db)
    }

    fn prices(&self) -> Result<Vec<Price>, Error> {
        Book::_prices(&self.db)
    }

    fn currencies(&self) -> Result<Vec<Commodity>, Error> {
        let result = self
            .commodities()?
            .into_iter()
            .filter(|x| x.namespace == "CURRENCY")
            .collect();
        Ok(result)
    }

    fn commodities(&self) -> Result<Vec<Commodity>, Error> {
        Book::_commodities(&self.db)
    }
}

impl _Account {
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
            Some(x) => x.children.iter().map(|x| x.as_element().unwrap()).collect(),
        };
        let placeholder = slots
            .iter()
            .find(|e| {
                e.get_child("key")
                    .and_then(|x| x.get_text())
                    .map(|x| x.into_owned())
                    .unwrap_or_else(|| String::from(""))
                    == "placeholder"
            })
            .and_then(|x| x.get_child("value"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| if x == "true" { 1 } else { 0 });

        _Account {
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

impl AccountT for Account {
    type DB = DB;

    fn splits(&self) -> Result<Vec<Split>, Error> {
        Book::_splits(&self.db).map(|x| {
            x.into_iter()
                .filter(|x| x.account_guid == self.guid)
                .collect()
        })
    }
    fn parent(&self) -> Option<Account> {
        Book::_accounts(&self.db)
            .map(|x| {
                x.into_iter().find(|x| {
                    x.guid.as_str() == self.parent_guid.as_ref().map_or("", |x| x.as_str())
                })
            })
            .unwrap_or(None)
    }
    fn children(&self) -> Result<Vec<Account>, Error> {
        Book::_accounts(&self.db).map(|x| {
            x.into_iter()
                .filter(|x| x.parent_guid.as_ref().map_or("", |x| x.as_str()) == self.guid.as_str())
                .collect()
        })
    }
    fn commodity(&self) -> Option<Commodity> {
        Book::_commodities(&self.db)
            .map(|x| {
                x.into_iter().find(|x| {
                    x.guid.as_str() == self.commodity_guid.as_ref().map_or("", |x| x.as_str())
                })
            })
            .unwrap_or(None)
    }

    fn balance(&self) -> Result<f64, Error> {
        let splits = self.splits()?;
        let mut net = splits.iter().fold(0.0, |acc, x| acc + x.quantity);

        for child in self.children()? {
            let child_net = child.balance()?;
            net += child_net;
        }

        let commodity = match self.commodity() {
            Some(commodity) => commodity,
            None => return Ok(net),
        };

        let currency = match self.parent() {
            Some(parent) => match parent.commodity() {
                Some(currency) => currency,
                None => return Ok(net),
            },
            None => return Ok(net),
        };

        match commodity.sell(&currency)? {
            Some(rate) => Ok(net * rate),
            None => Ok(net),
        }
    }
}

impl _Split {
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
            .unwrap_or_else(|| String::from(""));
        let action = e
            .get_child("action")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .unwrap_or_else(|| String::from(""));
        let reconcile_state = e
            .get_child("reconciled-state")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .unwrap_or_else(|| String::from(""));
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

        _Split {
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

impl SplitT for Split {
    type DB = DB;

    fn transaction(&self) -> Result<Transaction, Error> {
        Book::_transactions(&self.db).map(|x| {
            x.into_iter()
                .find(|x| x.guid == self.tx_guid)
                .expect("tx_guid must match one")
        })
    }

    fn account(&self) -> Result<Account, Error> {
        Book::_accounts(&self.db).map(|x| {
            x.into_iter()
                .find(|x| x.guid == self.account_guid)
                .expect("tx_guid must match one")
        })
    }
}

impl _Transaction {
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
            .unwrap_or_else(|| String::from(""));
        let post_date = e
            .get_child("date-posted")
            .and_then(|x| x.get_child("date"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z")
                    .expect("date-posted must be %Y-%m-%d %H:%M:%S%z")
            });
        let enter_date = e
            .get_child("date-entered")
            .and_then(|x| x.get_child("date"))
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| {
                chrono::NaiveDateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S%z")
                    .expect("date-entered must be %Y-%m-%d %H:%M:%S%z")
            });
        let description = e
            .get_child("description")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());

        _Transaction {
            guid,
            currency_guid,
            num,
            post_date,
            enter_date,
            description,
        }
    }
}

impl TransactionT for Transaction {
    type DB = DB;

    fn currency(&self) -> Result<Commodity, Error> {
        Book::_commodities(&self.db).map(|x| {
            x.into_iter()
                .find(|x| x.guid == self.currency_guid)
                .expect("tx_guid must match one")
        })
    }

    fn splits(&self) -> Result<Vec<Split>, Error> {
        Book::_splits(&self.db).map(|x| x.into_iter().filter(|x| x.tx_guid == self.guid).collect())
    }
}

impl _Price {
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

        _Price {
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

impl PriceT for Price {
    type DB = DB;

    fn commodity(&self) -> Result<Commodity, Error> {
        Book::_commodities(&self.db).map(|x| {
            x.into_iter()
                .find(|x| x.guid == self.commodity_guid)
                .expect("tx_guid must match one")
        })
    }

    fn currency(&self) -> Result<Commodity, Error> {
        Book::_commodities(&self.db).map(|x| {
            x.into_iter()
                .find(|x| x.guid == self.currency_guid)
                .expect("tx_guid must match one")
        })
    }
}

impl _Commodity {
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
        let quote_flag = e
            .get_child("quote_flag")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned())
            .map(|x| x.parse().expect("must be i32"))
            .unwrap_or(1);
        let quote_source = e
            .get_child("quote_source")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());
        let quote_tz = e
            .get_child("quote_tz")
            .and_then(|x| x.get_text())
            .map(|x| x.into_owned());

        _Commodity {
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

impl CommodityT for Commodity {
    type DB = DB;

    fn accounts(&self) -> Result<Vec<Account>, Error> {
        Book::_accounts(&self.db).map(|x| {
            x.into_iter()
                .filter(|x| {
                    x.commodity_guid.as_ref().map_or("", |x| x.as_str()) == self.guid.as_str()
                })
                .collect()
        })
    }

    fn transactions(&self) -> Result<Vec<Transaction>, Error> {
        Book::_transactions(&self.db).map(|x| {
            x.into_iter()
                .filter(|x| x.currency_guid == self.guid)
                .collect()
        })
    }

    fn as_commodity_prices(&self) -> Result<Vec<Price>, Error> {
        Book::_prices(&self.db).map(|x| {
            x.into_iter()
                .filter(|x| x.commodity_guid == self.guid)
                .collect()
        })
    }

    fn as_currency_prices(&self) -> Result<Vec<Price>, Error> {
        Book::_prices(&self.db).map(|x| {
            x.into_iter()
                .filter(|x| x.currency_guid == self.guid)
                .collect()
        })
    }

    fn as_commodity_or_currency_prices(&self) -> Result<Vec<Price>, Error> {
        Book::_prices(&self.db).map(|x| {
            x.into_iter()
                .filter(|x| x.commodity_guid == self.guid || x.currency_guid == self.guid)
                .collect()
        })
    }

    fn sell(&self, currency: &Commodity) -> Result<Option<f64>, Error> {
        if self.guid == currency.guid {
            return Ok(Some(1.0));
        }

        let mut prices: Vec<Price> = self
            .as_commodity_or_currency_prices()?
            .into_iter()
            .filter(|x| x.currency_guid == currency.guid || x.commodity_guid == currency.guid)
            .collect();
        prices.sort_by_key(|x| x.date);

        match prices.last() {
            Some(p) => {
                if self.guid == p.commodity_guid && currency.guid == p.currency_guid {
                    Ok(Some(p.value))
                } else if self.guid == p.currency_guid && currency.guid == p.commodity_guid {
                    Ok(Some(p.value_denom as f64 / p.value_num as f64))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    fn buy(&self, commodity: &Commodity) -> Result<Option<f64>, Error> {
        match commodity.sell(&self) {
            Ok(Some(value)) => Ok(Some(1.0 / value)),
            x => x,
        }
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

            let account = _Account::new_by_element(&e);

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
                _Split::new_by_element(String::from("6c8876003c4a6026e38e3afb67d6f2b1"), &e);

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

            let transaction = _Transaction::new_by_element(&e);

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

            let price = _Price::new_by_element(&e);

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

            let commodity = _Commodity::new_by_element(&e);

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
