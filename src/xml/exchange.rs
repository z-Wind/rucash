use super::wrap::DataWithPool;
use super::XMLPool;
use crate::model;
use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub(crate) struct Exchange {
    graph: HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>>,
    pool: XMLPool,
}

impl Exchange {
    fn new_graph(pool: XMLPool) -> HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>> {
        let prices: Vec<DataWithPool<model::Price>> = pool.prices(None);

        let mut graph: HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>> =
            HashMap::new();
        for p in prices {
            let commodity = &p.commodity().mnemonic;
            let currency = &p.currency().mnemonic;

            graph
                .entry(commodity.clone())
                .or_default()
                .entry(currency.clone())
                .and_modify(|e| {
                    if e.1 < p.date {
                        *e = (p.value(), p.date);
                    }
                })
                .or_insert((p.value(), p.date));

            graph
                .entry(currency.clone())
                .or_default()
                .entry(commodity.clone())
                .and_modify(|e| {
                    if e.1 < p.date {
                        *e = (num_traits::one::<crate::Num>() / p.value(), p.date);
                    }
                })
                .or_insert((num_traits::one::<crate::Num>() / p.value(), p.date));
        }

        graph
    }

    pub(crate) fn new(pool: XMLPool) -> Self {
        Self {
            graph: Self::new_graph(pool.clone()),
            pool,
        }
    }

    pub(crate) fn cal(
        &self,
        commodity: &DataWithPool<model::Commodity>,
        currency: &DataWithPool<model::Commodity>,
    ) -> Option<crate::Num> {
        let commodity = &commodity.mnemonic;
        let currency = &currency.mnemonic;
        if commodity == currency {
            return Some(num_traits::one());
        }

        let mut visited: HashSet<(&str, &str)> = HashSet::new();
        let mut queue: VecDeque<(&str, crate::Num, chrono::NaiveDateTime)> = VecDeque::new();
        queue.push_back((
            commodity,
            num_traits::one(),
            chrono::Local::now().naive_local(),
        ));

        while !queue.is_empty() {
            let n = queue.len();
            let mut done = false;
            for _ in 0..n {
                let (c, rate, date) = queue.pop_front().unwrap();
                if let Some(map) = self.graph.get(c) {
                    for (k, v) in map.iter() {
                        if visited.contains(&(c, k)) {
                            continue;
                        }
                        if k == currency {
                            done = true;
                        }
                        visited.insert((c, k));
                        visited.insert((k, c));

                        // println!("{} to {} = {:?}", c, k, v);
                        queue.push_back((k, rate * v.0, date.min(v.1)))
                    }
                }
            }
            // queue.iter().for_each(|x| println!("queue:{:?}", x));
            // println!("==============================");
            // println!("");
            if done {
                return Some(
                    queue
                        .into_iter()
                        .filter(|x| x.0 == currency)
                        .max_by_key(|x| x.2)
                        .map(|x| x.1)
                        .expect("must match"),
                );
            }
        }

        None
    }

    pub(crate) fn update(&mut self) {
        self.graph = Self::new_graph(self.pool.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    #[cfg(feature = "xml")]
    mod xml {
        use super::*;

        fn setup() -> crate::XMLBook {
            let uri: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );
            crate::XMLBook::new(uri).expect("right path")
        }

        #[test]
        fn test_exchange() {
            let book = setup();
            let mut exchange = Exchange::new(book.pool.clone());
            exchange.update();

            let from = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "ADF")
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.5, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(10, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(9, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "USD")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0 / 1.4, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(
                Decimal::new(10, 1) / Decimal::new(14, 1),
                exchange.cal(&from, &to).unwrap()
            );

            let from = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(9, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "USD")
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(
                f64,
                7.0 / 5.0 * 10.0 / 9.0,
                exchange.cal(&from, &to).unwrap()
            );
            #[cfg(feature = "decimal")]
            assert_eq!(
                (Decimal::new(7, 0) / Decimal::new(5, 0))
                    * (Decimal::new(10, 0) / Decimal::new(9, 0)),
                exchange.cal(&from, &to).unwrap()
            );

            let from = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.81, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(81, 2), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            let to = book
                .commodities()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0 / 0.81, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(
                Decimal::new(10, 1) / Decimal::new(81, 2),
                exchange.cal(&from, &to).unwrap()
            );
        }
    }
}
