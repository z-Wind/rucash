use super::XMLPool;
use crate::model;

use std::ops::Deref;

#[derive(Debug)]
pub struct DataWithPool<T> {
    content: T,
    pub(crate) pool: XMLPool,
}

impl<T> DataWithPool<T> {
    pub(crate) fn new(content: T, pool: XMLPool) -> Self
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
        let net: f64 = splits.iter().map(|x| x.quantity).sum::<f64>()
            + self
                .children()
                .iter()
                .map(|child| child.balance())
                .sum::<f64>();

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
        commodity.sell(self).map(|v| 1.0 / v)
    }
}
