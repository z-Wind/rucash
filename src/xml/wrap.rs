use super::XMLPool;
use crate::model::{self, Commodity};
use std::collections::HashSet;
use std::ops::Deref;

#[derive(Debug, Clone)]
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

    fn balance_into_currency(&self, currency: &DataWithPool<Commodity>) -> f64 {
        let mut net: f64 = self.splits().iter().map(|s| s.quantity).sum();
        let commodity = self.commodity().expect("must have commodity");

        for child in self.children() {
            let child_net = child.balance_into_currency(&commodity);
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

        net * rate
    }

    pub fn balance(&self) -> f64 {
        let mut net: f64 = self.splits().iter().map(|s| s.quantity).sum();

        let commodity = match self.commodity() {
            Some(commodity) => commodity,
            None => return net,
        };

        for child in self.children() {
            let child_net = child.balance_into_currency(&commodity);
            net += child_net;
        }
        // dbg!((&self.name, net));

        net
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

            for p in c.as_commodity_prices() {
                // println!(
                //     "{}: {} to {} = {}",
                //     rate * p.value,
                //     c.mnemonic,
                //     p.currency().mnemonic,
                //     p.value
                // );
                stack.push((p.currency(), rate * p.value, date.min(p.date)))
            }
            for p in c.as_currency_prices() {
                // println!(
                //     "{}: {} to {} = {}",
                //     rate * 1.0 / p.value,
                //     c.mnemonic,
                //     p.commodity().mnemonic,
                //     1.0 / p.value
                // );
                stack.push((p.commodity(), rate * 1.0 / p.value, date.min(p.date)))
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
    #[cfg(feature = "xml")]
    mod xml {
        use super::*;

        const URI: &str = r"tests\db\xml\complex_sample.gnucash";

        fn setup(uri: &str) -> crate::XMLBook {
            crate::XMLBook::new(uri).expect("right path")
        }

        #[test]
        fn test_exchange() {
            let book = setup(URI);

            let from = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "ADF")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "AED")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 1.5, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "FOO")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "FOO")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 1.0, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "FOO")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "AED")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 0.9, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "EUR")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "USD")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 1.0 / 1.4, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "AED")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "EUR")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 0.9, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "FOO")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "EUR")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 0.81, exchange(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "EUR")
                .next()
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .filter(|c| c.mnemonic == "FOO")
                .next()
                .unwrap();
            assert_approx_eq!(f64, 1.0 / 0.81, exchange(&from, &to).unwrap());
        }

        #[test]
        fn account() {
            let book = setup(URI);

            let account = book.account_by_name("Foo stock").unwrap();
            assert_eq!("Foo stock", account.name);
            assert_eq!(1, account.splits().len());
            assert_eq!("Broker", account.parent().unwrap().name);
            assert_eq!(0, account.children().len());
            assert_eq!("FOO", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 130.0, account.balance());

            let account = book.account_by_name("Cash").unwrap();
            assert_eq!("Cash", account.name);
            assert_eq!(3, account.splits().len());
            assert_eq!("Current", account.parent().unwrap().name);
            assert_eq!(0, account.children().len());
            assert_eq!("EUR", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 220.0, account.balance());

            let account = book.account_by_name("Mouvements").unwrap();
            assert_eq!("Mouvements", account.name);
            assert_eq!(0, account.splits().len());
            assert_eq!("Root Account", account.parent().unwrap().name);
            assert_eq!(2, account.children().len());
            assert_eq!("FOO", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 1351.4815, account.balance(), epsilon = 1e-4);

            let account = book.account_by_name("Asset").unwrap();
            assert_eq!("Asset", account.name);
            assert_eq!(0, account.splits().len());
            assert_eq!("Root Account", account.parent().unwrap().name);
            assert_eq!(3, account.children().len());
            assert_eq!("EUR", account.commodity().unwrap().mnemonic);
            assert_approx_eq!(f64, 24695.30, account.balance());
        }
    }
}
