use crate::kind::SQLKind;
use crate::model::{self, Commodity};
use futures::executor::block_on;
use std::collections::HashSet;
use std::ops::Deref;

#[derive(Debug, Clone)]
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

    fn balance_into_currency(
        &self,
        currency: &DataWithPool<Commodity>,
    ) -> Result<f64, sqlx::Error> {
        let mut net: f64 = self.splits()?.iter().map(|s| s.quantity).sum();
        let commodity = self.commodity().expect("must have commodity");

        for child in self.children()? {
            let child_net = child.balance_into_currency(&commodity)?;
            net += child_net;
        }

        let rate = commodity.sell(currency).unwrap_or_else(|| {
            panic!(
                "must have rate {} to {}",
                commodity.mnemonic, currency.mnemonic
            )
        });
        // dbg!((
        //     &commodity.mnemonic,
        //     &currency.mnemonic,
        //     rate,
        //     &self.name,
        //     net
        // ));

        Ok(net * rate)
    }

    pub fn balance(&self) -> Result<f64, sqlx::Error> {
        let mut net: f64 = self.splits()?.iter().map(|s| s.quantity).sum();

        let commodity = match self.commodity() {
            Some(commodity) => commodity,
            None => return Ok(net),
        };

        for child in self.children()? {
            let child_net = child.balance_into_currency(&commodity)?;
            net += child_net;
        }
        // dbg!((&self.name, net));

        Ok(net)
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

    pub fn sell(&self, currency: &DataWithPool<model::Commodity>) -> Option<f64> {
        // println!("{} to {}", self.mnemonic, currency.mnemonic);
        exchange(self, currency)
    }

    pub fn buy(&self, commodity: &DataWithPool<model::Commodity>) -> Option<f64> {
        // println!("{} to {}", commodity.mnemonic, self.mnemonic);
        exchange(commodity, self)
    }
}

fn exchange(
    commodity: &DataWithPool<model::Commodity>,
    currency: &DataWithPool<model::Commodity>,
) -> Option<f64> {
    fn leave_recent(
        mut stack: Vec<(DataWithPool<model::Commodity>, f64, chrono::NaiveDateTime)>,
    ) -> Vec<(DataWithPool<model::Commodity>, f64, chrono::NaiveDateTime)> {
        let mut result = Vec::new();
        // reverse sort
        stack.sort_unstable_by(|p1, p2| p2.2.cmp(&p1.2));

        let mut exist = HashSet::new();

        for x in stack {
            let key = x.0.guid.clone();
            if exist.contains(&key) {
                continue;
            }
            exist.insert(key);
            result.push(x)
        }

        result
    }

    let mut visited = HashSet::new();
    let mut stack: Vec<(DataWithPool<model::Commodity>, f64, chrono::NaiveDateTime)> = Vec::new();
    stack.push((
        commodity.clone(),
        1.0f64,
        chrono::Local::now().naive_local(),
    ));
    let mut n = stack.len();

    while n > 0 {
        for _ in 0..n {
            let (c, rate, date) = stack.pop().unwrap();
            if visited.contains(&c.guid) {
                continue;
            }

            visited.insert(c.guid.clone());

            for p in c.as_commodity_prices().ok()? {
                // println!(
                //     "{}: {} to {} = {}",
                //     rate * p.value,
                //     c.mnemonic,
                //     p.currency().ok()?.mnemonic,
                //     p.value
                // );
                stack.push((p.currency().ok()?, rate * p.value, date.min(p.date)))
            }
            for p in c.as_currency_prices().ok()? {
                // println!(
                //     "{}: {} to {} = {}",
                //     rate * 1.0 / p.value,
                //     c.mnemonic,
                //     p.commodity().ok()?.mnemonic,
                //     1.0 / p.value
                // );
                stack.push((p.commodity().ok()?, rate * 1.0 / p.value, date.min(p.date)))
            }
            // println!("==============================");
        }
        stack = leave_recent(stack);
        if let Some(rate) = stack
            .iter()
            .find(|x| x.0.guid == currency.guid)
            .map(|x| x.1)
        {
            return Some(rate);
        }
        // stack
        //     .iter()
        //     .for_each(|x| println!("{}: {} {:?}", x.0.mnemonic, x.1, x.2));
        // println!("==============================");
        // println!("==============================");

        n = stack.len();
    }

    None
}
#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        const URI: &str = r"sqlite://tests/db/sqlite/complex_sample.gnucash";
        type DB = sqlx::Sqlite;

        fn setup(uri: &str) -> crate::SqliteBook {
            let path = format!("sqlite://{}?mode=ro", uri);
            crate::SqliteBook::new(&path).expect("right path")
        }

        #[test]
        fn test_exchange() {
            let book = setup(URI);

            let from = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "ADF")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "AED")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 1.5, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "FOO")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "FOO")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 1.0, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "FOO")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "AED")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 0.9, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "EUR")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "USD")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 1.0 / 1.4, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "AED")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "EUR")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 0.9, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "FOO")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "EUR")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 0.81, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "EUR")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .unwrap()
                .into_iter()
                .filter(|c| c.mnemonic == "FOO")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 1.0 / 0.81, exchange(&from, &to).unwrap());
        }

        #[test]
        fn account() {
            let book = setup(URI);

            let account = book.account_by_name("Foo stock").unwrap().unwrap();
            assert_eq!("Foo stock", account.name);
            assert_eq!(1, account.splits().unwrap().len());
            assert_eq!("Broker", account.parent().unwrap().name);
            assert_eq!(0, account.children().unwrap().len());
            assert_eq!("FOO", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 130.0, account.balance().unwrap());

            let account = book.account_by_name("Cash").unwrap().unwrap();
            assert_eq!("Cash", account.name);
            assert_eq!(3, account.splits().unwrap().len());
            assert_eq!("Current", account.parent().unwrap().name);
            assert_eq!(0, account.children().unwrap().len());
            assert_eq!("EUR", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 220.0, account.balance().unwrap());

            let account = book.account_by_name("Mouvements").unwrap().unwrap();
            assert_eq!("Mouvements", account.name);
            assert_eq!(0, account.splits().unwrap().len());
            assert_eq!("Root Account", account.parent().unwrap().name);
            assert_eq!(2, account.children().unwrap().len());
            assert_eq!("FOO", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 1351.4815, account.balance().unwrap(), epsilon = 1e-4);

            let account = book.account_by_name("Asset").unwrap().unwrap();
            assert_eq!("Asset", account.name);
            assert_eq!(0, account.splits().unwrap().len());
            assert_eq!("Root Account", account.parent().unwrap().name);
            assert_eq!(3, account.children().unwrap().len());
            assert_eq!("EUR", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 24695.30, account.balance().unwrap());
        }
    }
}
