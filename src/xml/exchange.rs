use super::wrap::DataWithPool;
use super::XMLPool;
use crate::model;
use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

pub(crate) struct Exchange {
    graph: HashMap<String, HashMap<String, (f64, NaiveDateTime)>>,
}

impl Exchange {
    pub(crate) fn new(pool: XMLPool) -> Self {
        let prices: Vec<DataWithPool<model::Price>> = pool.prices();

        let mut graph: HashMap<String, HashMap<String, (f64, NaiveDateTime)>> = HashMap::new();
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
                        *e = (1.0 / p.value(), p.date);
                    }
                })
                .or_insert((1.0 / p.value(), p.date));
        }

        Self { graph }
    }

    pub(crate) fn cal(
        &self,
        commodity: &DataWithPool<model::Commodity>,
        currency: &DataWithPool<model::Commodity>,
    ) -> Option<f64> {
        let commodity = &commodity.mnemonic;
        let currency = &currency.mnemonic;
        if commodity == currency {
            return Some(1.0);
        }

        let mut visited: HashSet<(&str, &str)> = HashSet::new();
        let mut queue: VecDeque<(&str, f64, chrono::NaiveDateTime)> = VecDeque::new();
        queue.push_back((commodity, 1.0f64, chrono::Local::now().naive_local()));

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
}

#[cfg(test)]
mod tests {
    use super::*;
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
        fn test_exchange() {
            let book = setup(URI);
            let exchange = Exchange::new(book.pool.clone());

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
            assert_approx_eq!(f64, 1.5, exchange.cal(&from, &to).unwrap());

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
            assert_approx_eq!(f64, 1.0, exchange.cal(&from, &to).unwrap());

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
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());

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
            assert_approx_eq!(f64, 1.0 / 1.4, exchange.cal(&from, &to).unwrap());

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
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());

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
            assert_approx_eq!(f64, 0.81, exchange.cal(&from, &to).unwrap());

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
            assert_approx_eq!(f64, 1.0 / 0.81, exchange.cal(&from, &to).unwrap());
        }
    }
}
