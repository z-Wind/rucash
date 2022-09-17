use crate::kind::SQLKind;
use crate::model;
use futures::executor::block_on;
use std::ops::Deref;

#[derive(Debug)]
pub struct DataWithPool<T> {
    content: T,
    kind: SQLKind,
    pub pool: sqlx::AnyPool,
}

impl<T> DataWithPool<T> {
    pub(crate) fn new(content: T, kind: SQLKind, pool: sqlx::AnyPool) -> Self
    where
        T: model::NullNone,
    {
        Self {
            content: content.null_none(),
            kind,
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
    pub fn splits(&self) -> Result<Vec<DataWithPool<model::Split>>, sqlx::Error> {
        block_on(async {
            model::Split::query_by_account_guid(&self.guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn parent(&self) -> Option<DataWithPool<model::Account>> {
        let guid = self.parent_guid.as_ref()?;
        block_on(async {
            model::Account::query_by_guid(guid, self.kind)
                .fetch_optional(&self.pool)
                .await
                .unwrap()
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }

    pub fn children(&self) -> Result<Vec<DataWithPool<model::Account>>, sqlx::Error> {
        block_on(async {
            model::Account::query_by_parent_guid(&self.guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn commodity(&self) -> Option<DataWithPool<model::Commodity>> {
        let guid = self.commodity_guid.as_ref()?;
        block_on(async {
            model::Commodity::query_by_guid(guid, self.kind)
                .fetch_optional(&self.pool)
                .await
                .unwrap()
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }

    pub fn balance(&self) -> Result<f64, sqlx::Error> {
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

impl DataWithPool<model::Split> {
    pub fn transaction(&self) -> Result<DataWithPool<model::Transaction>, sqlx::Error> {
        let guid = &self.tx_guid;
        block_on(async {
            model::Transaction::query_by_guid(guid, self.kind)
                .fetch_one(&self.pool)
                .await
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }

    pub fn account(&self) -> Result<DataWithPool<model::Account>, sqlx::Error> {
        let guid = &self.account_guid;
        block_on(async {
            model::Account::query_by_guid(guid, self.kind)
                .fetch_one(&self.pool)
                .await
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }
}

impl DataWithPool<model::Transaction> {
    pub fn currency(&self) -> Result<DataWithPool<model::Commodity>, sqlx::Error> {
        let guid = &self.currency_guid;
        block_on(async {
            model::Commodity::query_by_guid(guid, self.kind)
                .fetch_one(&self.pool)
                .await
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }

    pub fn splits(&self) -> Result<Vec<DataWithPool<model::Split>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Split::query_by_tx_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }
}

impl DataWithPool<model::Price> {
    pub fn commodity(&self) -> Result<DataWithPool<model::Commodity>, sqlx::Error> {
        let guid = &self.commodity_guid;
        block_on(async {
            model::Commodity::query_by_guid(guid, self.kind)
                .fetch_one(&self.pool)
                .await
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }

    pub fn currency(&self) -> Result<DataWithPool<model::Commodity>, sqlx::Error> {
        let guid = &self.currency_guid;
        block_on(async {
            model::Commodity::query_by_guid(guid, self.kind)
                .fetch_one(&self.pool)
                .await
        })
        .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
    }
}

impl DataWithPool<model::Commodity> {
    pub fn accounts(&self) -> Result<Vec<DataWithPool<model::Account>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Account::query_by_commodity_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn transactions(&self) -> Result<Vec<DataWithPool<model::Transaction>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Transaction::query_by_currency_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn as_commodity_prices(&self) -> Result<Vec<DataWithPool<model::Price>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Price::query_by_commodity_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn as_currency_prices(&self) -> Result<Vec<DataWithPool<model::Price>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Price::query_by_currency_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn as_commodity_or_currency_prices(
        &self,
    ) -> Result<Vec<DataWithPool<model::Price>>, sqlx::Error> {
        let guid = &self.guid;
        block_on(async {
            model::Price::query_by_commodity_or_currency_guid(guid, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn sell(
        &self,
        currency: &DataWithPool<model::Commodity>,
    ) -> Result<Option<f64>, sqlx::Error> {
        if self.guid == currency.guid {
            return Ok(Some(1.0));
        }

        let mut prices: Vec<DataWithPool<model::Price>> = self
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

    pub fn buy(
        &self,
        commodity: &DataWithPool<model::Commodity>,
    ) -> Result<Option<f64>, sqlx::Error> {
        match commodity.sell(self) {
            Ok(Some(value)) => Ok(Some(1.0 / value)),
            x => x,
        }
    }
}
