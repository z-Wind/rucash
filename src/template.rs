use std::ops::Deref;
use std::rc::Rc;

pub use super::model::Account as _Account;
pub use super::model::Commodity as _Commodity;
pub use super::model::Price as _Price;
pub use super::model::Split as _Split;
pub use super::model::Transaction as _Transaction;

pub trait Consistency {
    fn consistency(self) -> Self;
}

#[derive(Debug)]
pub struct Book<DB> {
    pub db: Rc<DB>,
}

#[derive(Debug)]
pub struct Item<T, DB> {
    content: T,
    pub db: Rc<DB>,
}

impl<T, DB> Item<T, DB> {
    pub fn new(content: T, db: &Rc<DB>) -> Self
    where
        T: Consistency,
    {
        Self {
            content: content.consistency(),
            db: Rc::clone(&db),
        }
    }

    pub fn content(&self) -> &T {
        &self.content
    }
}

impl<T, DB> Deref for Item<T, DB> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.content
    }
}

impl<T, DB> PartialEq for Item<T, DB>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

impl<T, DB> PartialOrd for Item<T, DB>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.content.partial_cmp(&other.content)
    }
}

pub trait BookT {
    type DB;

    fn new(uri: &str) -> Result<Book<Self::DB>, Box<dyn std::error::Error>>;

    fn accounts(&self) -> Result<Vec<Item<_Account, Self::DB>>, Box<dyn std::error::Error>>;

    fn accounts_contains_name(
        &self,
        name: &str,
    ) -> Result<Vec<Item<_Account, Self::DB>>, Box<dyn std::error::Error>> {
        let name = name.to_lowercase();
        self.accounts().map(|x| {
            x.into_iter()
                .filter(|x| x.name.to_lowercase().contains(&name))
                .collect()
        })
    }

    fn account_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Item<_Account, Self::DB>>, Box<dyn std::error::Error>> {
        self.accounts_contains_name(name).map(|mut x| x.pop())
    }

    fn splits(&self) -> Result<Vec<Item<_Split, Self::DB>>, Box<dyn std::error::Error>>;

    fn transactions(&self)
        -> Result<Vec<Item<_Transaction, Self::DB>>, Box<dyn std::error::Error>>;

    fn prices(&self) -> Result<Vec<Item<_Price, Self::DB>>, Box<dyn std::error::Error>>;

    fn currencies(&self) -> Result<Vec<Item<_Commodity, Self::DB>>, Box<dyn std::error::Error>>;

    fn commodities(&self) -> Result<Vec<Item<_Commodity, Self::DB>>, Box<dyn std::error::Error>>;
}
pub trait AccountT {
    type DB;

    fn splits(&self) -> Result<Vec<Item<_Split, Self::DB>>, Box<dyn std::error::Error>>;
    fn parent(&self) -> Option<Item<_Account, Self::DB>>;
    fn children(&self) -> Result<Vec<Item<_Account, Self::DB>>, Box<dyn std::error::Error>>;
    fn commodity(&self) -> Option<Item<_Commodity, Self::DB>>;

    fn balance(&self) -> Result<f64, Box<dyn std::error::Error>>;
}

pub trait SplitT {
    type DB;

    fn transaction(&self) -> Result<Item<_Transaction, Self::DB>, Box<dyn std::error::Error>>;

    fn account(&self) -> Result<Item<_Account, Self::DB>, Box<dyn std::error::Error>>;
}

pub trait TransactionT {
    type DB;

    fn currency(&self) -> Result<Item<_Commodity, Self::DB>, Box<dyn std::error::Error>>;

    fn splits(&self) -> Result<Vec<Item<_Split, Self::DB>>, Box<dyn std::error::Error>>;
}

pub trait PriceT {
    type DB;

    fn commodity(&self) -> Result<Item<_Commodity, Self::DB>, Box<dyn std::error::Error>>;

    fn currency(&self) -> Result<Item<_Commodity, Self::DB>, Box<dyn std::error::Error>>;
}

pub trait CommodityT {
    type DB;

    fn accounts(&self) -> Result<Vec<Item<_Account, Self::DB>>, Box<dyn std::error::Error>>;

    fn transactions(&self)
        -> Result<Vec<Item<_Transaction, Self::DB>>, Box<dyn std::error::Error>>;

    fn as_commodity_prices(
        &self,
    ) -> Result<Vec<Item<_Price, Self::DB>>, Box<dyn std::error::Error>>;

    fn as_currency_prices(&self)
        -> Result<Vec<Item<_Price, Self::DB>>, Box<dyn std::error::Error>>;

    fn as_commodity_or_currency_prices(
        &self,
    ) -> Result<Vec<Item<_Price, Self::DB>>, Box<dyn std::error::Error>>;

    fn sell(
        &self,
        currency: &Item<_Commodity, Self::DB>,
    ) -> Result<Option<f64>, Box<dyn std::error::Error>>;

    fn buy(
        &self,
        commodity: &Item<_Commodity, Self::DB>,
    ) -> Result<Option<f64>, Box<dyn std::error::Error>>;
}
