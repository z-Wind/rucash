use chrono::NaiveDateTime;
#[cfg(not(feature = "decimal"))]
use num_traits::Zero;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use crate::error::Error;
use crate::model::{Commodity, Price};
use crate::query::{CommodityQ, PriceQ, Query};

#[derive(Debug, Clone)]
pub(crate) struct Exchange {
    graph: HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>>,
}

impl Exchange {
    pub(crate) async fn new<Q>(query: Arc<Q>) -> Result<Self, Error>
    where
        Q: Query,
    {
        Ok(Self {
            graph: Self::new_graph(query.clone()).await?,
        })
    }

    async fn new_graph<Q>(
        query: Arc<Q>,
    ) -> Result<HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>>, Error>
    where
        Q: Query,
    {
        let prices: Vec<Price<Q>> = PriceQ::all(&*query)
            .await?
            .into_iter()
            .map(|x| Price::from_with_query(&x, query.clone()))
            .collect();

        let commodities_map: HashMap<String, String> = CommodityQ::all(&*query)
            .await?
            .into_iter()
            .map(|x| {
                let c = Commodity::from_with_query(&x, query.clone());
                (c.guid.clone(), c.mnemonic.clone())
            })
            .collect();

        let mut graph: HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>> =
            HashMap::new();

        for p in prices {
            let commodity = commodities_map
                .get(&p.commodity_guid)
                .ok_or(Error::GuidNotFound {
                    model: "Commodity".to_string(),
                    guid: p.commodity_guid.clone(),
                })?;
            let currency = commodities_map
                .get(&p.currency_guid)
                .ok_or(Error::GuidNotFound {
                    model: "Commodity".to_string(),
                    guid: p.currency_guid.clone(),
                })?;

            if p.value.is_zero() {
                println!(
                    "Warning: ignore {} {commodity}/{currency} in exchange graph, because the value is zero.",
                    p.datetime
                );
                continue;
            }

            graph
                .entry(commodity.clone())
                .or_default()
                .entry(currency.clone())
                .and_modify(|e| {
                    if e.1 < p.datetime {
                        *e = (p.value, p.datetime);
                    }
                })
                .or_insert((p.value, p.datetime));

            graph
                .entry(currency.clone())
                .or_default()
                .entry(commodity.clone())
                .and_modify(|e| {
                    if e.1 < p.datetime {
                        *e = (num_traits::one::<crate::Num>() / p.value, p.datetime);
                    }
                })
                .or_insert((num_traits::one::<crate::Num>() / p.value, p.datetime));
        }

        Ok(graph)
    }

    pub(crate) fn cal<Q>(
        &self,
        commodity: &Commodity<Q>,
        currency: &Commodity<Q>,
    ) -> Option<crate::Num>
    where
        Q: Query,
    {
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
                let (c, r, date) = queue.pop_front().unwrap();
                if let Some(map) = self.graph.get(c) {
                    for (k, v) in map {
                        if visited.contains(&(c, k)) {
                            continue;
                        }
                        if k == currency {
                            done = true;
                        }
                        visited.insert((c, k));
                        visited.insert((k, c));

                        // println!("{} to {} = {:?}", c, k, v);
                        queue.push_back((k, r * v.0, date.min(v.1)));
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

    pub(crate) async fn update<Q>(&mut self, query: Arc<Q>) -> Result<(), Error>
    where
        Q: Query,
    {
        self.graph = Self::new_graph(query).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Book;

    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        use crate::SQLiteQuery;

        use pretty_assertions::assert_eq;

        #[allow(clippy::unused_async)]
        async fn setup() -> SQLiteQuery {
            let uri: &str = &format!(
                "{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            println!("work_dir: {:?}", std::env::current_dir());
            SQLiteQuery::new(uri).unwrap()
        }

        #[tokio::test]
        #[allow(clippy::too_many_lines)]
        async fn test_exchange() {
            let query = setup().await;
            let book = Book::new(query.clone()).await.unwrap();
            let query = Arc::new(query);
            let mut exchange = Exchange::new(query.clone()).await.unwrap();
            exchange.update(query).await.expect("ok");

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "ADF")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            assert_eq!(from.mnemonic, "ADF");
            assert_eq!(to.mnemonic, "AED");
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.5, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(10, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(9, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(9, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "USD")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.81, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(81, 2), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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

    #[cfg(feature = "mysql")]
    mod mysql {
        use super::*;
        use crate::MySQLQuery;

        use pretty_assertions::assert_eq;

        async fn setup() -> MySQLQuery {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            MySQLQuery::new(uri)
                .await
                .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"))
        }

        #[tokio::test]
        #[allow(clippy::too_many_lines)]
        async fn test_exchange() {
            let query = setup().await;
            let book = Book::new(query.clone()).await.unwrap();
            let query = Arc::new(query);
            let mut exchange = Exchange::new(query.clone()).await.unwrap();
            exchange.update(query).await.expect("ok");

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "ADF")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            assert_eq!(from.mnemonic, "ADF");
            assert_eq!(to.mnemonic, "AED");
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.5, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(10, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(9, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(9, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "USD")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.81, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(81, 2), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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

    #[cfg(feature = "postgresql")]
    mod postgresql {
        use super::*;

        use crate::PostgreSQLQuery;

        use pretty_assertions::assert_eq;

        async fn setup() -> PostgreSQLQuery {
            let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            PostgreSQLQuery::new(uri)
                .await
                .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"))
        }

        #[tokio::test]
        #[allow(clippy::too_many_lines)]
        async fn test_exchange() {
            let query = setup().await;
            let book = Book::new(query.clone()).await.unwrap();
            let query = Arc::new(query);
            let mut exchange = Exchange::new(query.clone()).await.unwrap();
            exchange.update(query).await.expect("ok");

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "ADF")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            assert_eq!(from.mnemonic, "ADF");
            assert_eq!(to.mnemonic, "AED");
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.5, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(10, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(9, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(9, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "USD")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.81, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(81, 2), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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

    #[cfg(feature = "xml")]
    mod xml {
        use super::*;
        use crate::XMLQuery;

        use pretty_assertions::assert_eq;

        fn setup() -> XMLQuery {
            let path: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            println!("work_dir: {:?}", std::env::current_dir());
            XMLQuery::new(path).unwrap()
        }

        #[tokio::test]
        #[allow(clippy::too_many_lines)]
        async fn test_exchange() {
            let query = setup();
            let book = Book::new(query.clone()).await.unwrap();
            let query = Arc::new(query);
            let mut exchange = Exchange::new(query.clone()).await.unwrap();
            exchange.update(query).await.expect("ok");

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "ADF")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            assert_eq!(from.mnemonic, "ADF");
            assert_eq!(to.mnemonic, "AED");
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.5, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(15, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 1.0, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(10, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(9, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "AED")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.9, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(9, 1), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "USD")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "FOO")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            #[cfg(not(feature = "decimal"))]
            assert_approx_eq!(f64, 0.81, exchange.cal(&from, &to).unwrap());
            #[cfg(feature = "decimal")]
            assert_eq!(Decimal::new(81, 2), exchange.cal(&from, &to).unwrap());

            let from = book
                .commodities()
                .await
                .unwrap()
                .into_iter()
                .find(|c| c.mnemonic == "EUR")
                .unwrap();
            let to = book
                .commodities()
                .await
                .unwrap()
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
