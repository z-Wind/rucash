use super::Exchange;
use super::XMLPool;
use crate::model::{self, Commodity};
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
            .expect("account_guid must match one")
    }
}

impl DataWithPool<model::Transaction> {
    pub fn currency(&self) -> DataWithPool<model::Commodity> {
        self.pool
            .commodities()
            .into_iter()
            .find(|x| x.guid == self.currency_guid)
            .expect("currency_guid must match one")
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
            .expect("commodity_guid must match one")
    }

    pub fn currency(&self) -> DataWithPool<model::Commodity> {
        self.pool
            .commodities()
            .into_iter()
            .find(|x| x.guid == self.currency_guid)
            .expect("currency_guid must match one")
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
        let exchange = Exchange::new(self.pool.clone());
        exchange.cal(self, currency)
    }

    pub fn buy(&self, commodity: &DataWithPool<model::Commodity>) -> Option<f64> {
        // println!("{} to {}", commodity.mnemonic, self.mnemonic);
        let exchange = Exchange::new(self.pool.clone());
        exchange.cal(commodity, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use float_cmp::assert_approx_eq;

    #[cfg(feature = "xml")]
    mod xml {
        use super::*;

        const URI: &str = r"tests/db/xml/complex_sample.gnucash";

        fn setup(uri: &str) -> crate::XMLBook {
            println!("work_dir: {:?}", std::env::current_dir());
            crate::XMLBook::new(uri).expect("right path")
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

        #[test]
        fn split() {
            let book = setup(URI);
            let split = book
                .splits()
                .into_iter()
                .find(|s| s.guid == "de832fe97e37811a7fff7e28b3a43425")
                .unwrap();

            assert_eq!(split.transaction().guid, "6c8876003c4a6026e38e3afb67d6f2b1");
            assert_eq!(split.transaction().description, Some("income 1".into()));
            assert_eq!(
                split.transaction().post_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );
            assert_eq!(
                split.transaction().enter_date,
                Some(
                    NaiveDateTime::parse_from_str("2014-12-25 10:08:15", "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                )
            );

            assert_eq!(split.account().guid, "93fc043c3062aaa1297b30e543d2cd0d",);
            assert_eq!(split.account().name, "Cash",);
        }

        #[test]
        fn transaction() {
            let book = setup(URI);
            let transaction = book
                .transactions()
                .into_iter()
                .find(|t| t.description == Some("buy foo".into()))
                .unwrap();

            assert_eq!(transaction.currency().mnemonic, "EUR");
            assert_eq!(transaction.splits().len(), 4);
        }

        #[test]
        fn price() {
            let book = setup(URI);
            let price = book
                .prices()
                .into_iter()
                .find(|p| p.guid == "0d6684f44fb018e882de76094ed9c433")
                .unwrap();

            assert_eq!(price.commodity().mnemonic, "ADF");
            assert_eq!(price.currency().mnemonic, "AED");
        }

        #[test]
        fn commodity() {
            let book = setup(URI);
            let commodity = book
                .commodities()
                .into_iter()
                .find(|p| p.mnemonic == "EUR")
                .unwrap();

            assert_eq!(commodity.accounts().len(), 14);
            assert_eq!(commodity.transactions().len(), 11);
            assert_eq!(commodity.as_commodity_prices().len(), 1);
            assert_eq!(commodity.as_currency_prices().len(), 2);
            assert_eq!(commodity.as_commodity_or_currency_prices().len(), 3);

            let currency = book
                .commodities()
                .into_iter()
                .find(|p| p.mnemonic == "FOO")
                .unwrap();
            assert_approx_eq!(f64, 1.2345679012345678, currency.buy(&commodity).unwrap());
            assert_approx_eq!(f64, 1.2345679012345678, commodity.sell(&currency).unwrap());
        }
    }
}
