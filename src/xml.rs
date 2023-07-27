pub mod error;
mod exchange;
pub mod wrap;

use exchange::Exchange;
use itertools::Itertools;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, RwLock};
use wrap::DataWithPool;
use xmltree::Element;

use super::model;
use crate::XMLError;
use flate2::read::GzDecoder;

#[derive(Debug)]
pub(crate) struct XMLPool(pub(crate) Arc<Element>);

impl Clone for XMLPool {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl XMLPool {
    /// read gnucash xml file in gzip
    fn new(uri: &str) -> Result<Self, XMLError> {
        let f = File::open(uri)?;
        let mut d = GzDecoder::new(f);
        let mut data = String::new();
        d.read_to_string(&mut data)?;

        let mut root: Element = Element::parse(data.as_bytes())?;
        root = root
            .take_child("book")
            .ok_or(XMLError::NoBook(uri.to_string()))?;

        Ok(Self(Arc::new(root)))
    }

    fn accounts(
        &self,
        exchange_graph: &Option<Arc<RwLock<Exchange>>>,
    ) -> Vec<DataWithPool<model::Account>> {
        self.0
            .children
            .iter()
            .filter_map(xmltree::XMLNode::as_element)
            .filter(|e| e.name == "account")
            .map(|e| {
                let content = model::Account::new_by_element(e);
                DataWithPool::<model::Account>::new(content, self.clone(), exchange_graph.clone())
            })
            .collect()
    }

    fn splits(
        &self,
        exchange_graph: &Option<Arc<RwLock<Exchange>>>,
    ) -> Vec<DataWithPool<model::Split>> {
        self.0
            .children
            .iter()
            .filter_map(xmltree::XMLNode::as_element)
            .filter(|e| e.name == "transaction")
            .flat_map(|e| {
                let tx_guid = e
                    .get_child("id")
                    .expect("id")
                    .get_text()
                    .expect("text")
                    .into_owned();
                let exchange_graph = exchange_graph.clone();
                e.get_child("splits")
                    .expect("splits")
                    .children
                    .iter()
                    .filter_map(xmltree::XMLNode::as_element)
                    .map(move |e| {
                        let content = model::Split::new_by_element(tx_guid.clone(), e);
                        DataWithPool::<model::Split>::new(
                            content,
                            self.clone(),
                            exchange_graph.clone(),
                        )
                    })
            })
            .collect()
    }

    fn transactions(
        &self,
        exchange_graph: &Option<Arc<RwLock<Exchange>>>,
    ) -> Vec<DataWithPool<model::Transaction>> {
        self.0
            .children
            .iter()
            .filter_map(xmltree::XMLNode::as_element)
            .filter(|e| e.name == "transaction")
            .map(|e| {
                let content = model::Transaction::new_by_element(e);
                DataWithPool::<model::Transaction>::new(
                    content,
                    self.clone(),
                    exchange_graph.clone(),
                )
            })
            .collect()
    }

    fn prices(
        &self,
        exchange_graph: &Option<Arc<RwLock<Exchange>>>,
    ) -> Vec<DataWithPool<model::Price>> {
        match self.0.get_child("pricedb") {
            None => Vec::new(),
            Some(node) => node
                .children
                .iter()
                .filter_map(xmltree::XMLNode::as_element)
                .filter(|e| e.name == "price")
                .map(|e| {
                    let content = model::Price::new_by_element(e);
                    DataWithPool::<model::Price>::new(content, self.clone(), exchange_graph.clone())
                })
                .collect(),
        }
    }

    fn commodities(
        &self,
        exchange_graph: &Option<Arc<RwLock<Exchange>>>,
    ) -> Vec<DataWithPool<model::Commodity>> {
        let mut commodities: Vec<DataWithPool<model::Commodity>> = self
            .0
            .children
            .iter()
            .filter_map(xmltree::XMLNode::as_element)
            .filter(|e| e.name == "commodity")
            .map(|e| {
                let content = model::Commodity::new_by_element(e);
                DataWithPool::<model::Commodity>::new(content, self.clone(), exchange_graph.clone())
            })
            .collect();

        let mut prices: Vec<DataWithPool<model::Commodity>> = match self.0.get_child("pricedb") {
            None => Vec::new(),
            Some(node) => node
                .children
                .iter()
                .filter_map(xmltree::XMLNode::as_element)
                .filter(|e| e.name == "price")
                .flat_map(|e| {
                    let cmdty = e.get_child("commodity").expect("must exist");
                    let content = model::Commodity::new_by_element(cmdty);
                    let cmdty = DataWithPool::<model::Commodity>::new(
                        content,
                        self.clone(),
                        exchange_graph.clone(),
                    );

                    let crncy = e.get_child("currency").expect("must exist");
                    let content = model::Commodity::new_by_element(crncy);
                    let crncy = DataWithPool::<model::Commodity>::new(
                        content,
                        self.clone(),
                        exchange_graph.clone(),
                    );

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
pub struct Book {
    pub(crate) pool: XMLPool,
    exchange_graph: Option<Arc<RwLock<Exchange>>>,
}

impl Book {
    /// read gnucash xml file in gzip
    pub fn new(uri: &str) -> Result<Self, XMLError> {
        let pool = XMLPool::new(uri)?;
        let exchange_graph = Some(Arc::new(RwLock::new(Exchange::new(pool.clone()))));

        Ok(Self {
            pool,
            exchange_graph,
        })
    }

    #[must_use]
    pub fn accounts(&self) -> Vec<DataWithPool<model::Account>> {
        self.pool.accounts(&self.exchange_graph)
    }

    #[must_use]
    pub fn account_by_name(&self, name: &str) -> Option<DataWithPool<model::Account>> {
        self.accounts_contains_name(name).pop()
    }

    #[must_use]
    pub fn accounts_contains_name(&self, name: &str) -> Vec<DataWithPool<model::Account>> {
        self.accounts()
            .into_iter()
            .filter(|x| x.name.to_lowercase().contains(&name.to_lowercase()))
            .collect()
    }

    #[must_use]
    pub fn splits(&self) -> Vec<DataWithPool<model::Split>> {
        self.pool.splits(&self.exchange_graph)
    }

    #[must_use]
    pub fn transactions(&self) -> Vec<DataWithPool<model::Transaction>> {
        self.pool.transactions(&self.exchange_graph)
    }

    #[must_use]
    pub fn prices(&self) -> Vec<DataWithPool<model::Price>> {
        self.pool.prices(&self.exchange_graph)
    }

    #[must_use]
    pub fn commodities(&self) -> Vec<DataWithPool<model::Commodity>> {
        self.pool.commodities(&self.exchange_graph)
    }

    #[must_use]
    pub fn currencies(&self) -> Vec<DataWithPool<model::Commodity>> {
        self.commodities()
            .into_iter()
            .filter(|x| x.namespace == "CURRENCY")
            .collect()
    }
}
