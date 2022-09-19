mod exchange;
pub mod wrap;

use super::model;
use flate2::read::GzDecoder;

use exchange::Exchange;
use itertools::Itertools;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use wrap::DataWithPool;
use xmltree::Element;

#[derive(Debug)]
pub(crate) struct XMLPool(pub(crate) Arc<Element>);

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
    pub(crate) pool: XMLPool,
}

impl XMLBook {
    /// read gnucash xml file in gzip
    pub fn new(uri: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = XMLPool::new(uri)?;
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
