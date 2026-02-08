use chrono::NaiveDateTime;
#[cfg(not(feature = "decimal"))]
use num_traits::Zero;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tracing::instrument;

use crate::error::Error;
use crate::model::{Commodity, Price};
use crate::query::{CommodityQ, PriceQ, Query};

type Graph = HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>>;

#[derive(Debug, Clone)]
pub(crate) struct Exchange {
    graph: HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>>,
}

impl Exchange {
    #[instrument(skip(query))]
    pub(crate) async fn new<Q>(query: Arc<Q>) -> Result<Self, Error>
    where
        Q: Query,
    {
        tracing::debug!("creating new exchange instance");
        let graph = Self::new_graph(query.clone())
            .await
            .inspect_err(|e| tracing::error!("failed to create exchange graph: {e}"))?;
        tracing::info!(nodes = graph.len(), "exchange graph created successfully");
        Ok(Self { graph })
    }

    #[instrument(skip(query))]
    async fn new_graph<Q>(query: Arc<Q>) -> Result<Graph, Error>
    where
        Q: Query,
    {
        tracing::debug!("building exchange graph from prices and commodities");

        let prices: Vec<Price<Q>> = PriceQ::all(&*query)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch prices: {e}"))?
            .into_iter()
            .map(|x| Price::from_with_query(&x, query.clone()))
            .collect();
        tracing::debug!(price_count = prices.len(), "prices loaded");

        let commodities_map: HashMap<String, String> = CommodityQ::all(&*query)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch commodities: {e}"))?
            .into_iter()
            .map(|x| {
                let c = Commodity::from_with_query(&x, query.clone());
                (c.guid.clone(), c.mnemonic.clone())
            })
            .collect();
        tracing::debug!(
            commodity_count = commodities_map.len(),
            "commodities loaded"
        );

        let mut graph: HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>> =
            HashMap::new();

        for p in prices {
            let commodity =
                commodities_map
                    .get(&p.commodity_guid)
                    .ok_or_else(|| Error::GuidNotFound {
                        model: "Commodity".to_string(),
                        guid: p.commodity_guid.clone(),
                    })?;
            let currency =
                commodities_map
                    .get(&p.currency_guid)
                    .ok_or_else(|| Error::GuidNotFound {
                        model: "Commodity".to_string(),
                        guid: p.currency_guid.clone(),
                    })?;

            if p.value.is_zero() {
                tracing::warn!(
                    datetime = %p.datetime,
                    commodity = commodity,
                    currency = currency,
                    "ignoring price with zero value in exchange graph"
                );
                continue;
            }

            graph
                .entry(commodity.clone())
                .or_default()
                .entry(currency.clone())
                .and_modify(|e| {
                    if e.1 < p.datetime {
                        tracing::debug!(
                            commodity = commodity,
                            currency = currency,
                            old_date = %e.1,
                            new_date = %p.datetime,
                            "updating price to newer entry"
                        );
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

        tracing::info!(
            graph_nodes = graph.len(),
            total_edges = graph.values().map(HashMap::len).sum::<usize>(),
            "exchange graph built successfully"
        );
        Ok(graph)
    }

    #[instrument(skip(self, commodity, currency), fields(
        commodity = %commodity.mnemonic,
        currency = %currency.mnemonic
    ))]
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
            tracing::debug!("same commodity and currency, returning 1.0");
            return Some(num_traits::one());
        }

        tracing::debug!("searching for exchange path using BFS");
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

                        tracing::trace!(from = c, to = k, rate = ?v.0, date = %v.1, "exploring edge");
                        queue.push_back((k, r * v.0, date.min(v.1)));
                    }
                }
            }
            if done {
                let result = queue
                    .into_iter()
                    .filter(|x| x.0 == currency)
                    .max_by_key(|x| x.2)
                    .map(|x| x.1)
                    .expect("must match");
                tracing::info!(?result, "exchange path found");
                return Some(result);
            }
        }

        tracing::warn!("no exchange path found between commodities");
        None
    }

    #[instrument(skip(self, query))]
    pub(crate) async fn update<Q>(&mut self, query: Arc<Q>) -> Result<(), Error>
    where
        Q: Query,
    {
        tracing::debug!("updating exchange graph");
        self.graph = Self::new_graph(query)
            .await
            .inspect_err(|e| tracing::error!("failed to rebuild exchange graph: {e}"))?;
        tracing::info!("exchange graph updated successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    use crate::Book;

    use super::*;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::SQLiteQuery;

        use super::*;

        #[allow(clippy::unused_async)]
        async fn setup() -> SQLiteQuery {
            let uri: &str = &format!(
                "{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            tracing::info!("work_dir: {:?}", std::env::current_dir());
            SQLiteQuery::new(uri).unwrap()
        }

        #[test(tokio::test)]
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
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::MySQLQuery;

        use super::*;

        async fn setup() -> MySQLQuery {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            MySQLQuery::new(uri)
                .await
                .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"))
        }

        #[test(tokio::test)]
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
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::PostgreSQLQuery;

        use super::*;

        async fn setup() -> PostgreSQLQuery {
            let uri = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
            PostgreSQLQuery::new(uri)
                .await
                .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"))
        }

        #[test(tokio::test)]
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
        use pretty_assertions::assert_eq;
        use test_log::test;

        use crate::XMLQuery;

        use super::*;

        fn setup() -> XMLQuery {
            let path: &str = &format!(
                "{}/tests/db/xml/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            tracing::info!("work_dir: {:?}", std::env::current_dir());
            XMLQuery::new(path).unwrap()
        }

        #[test(tokio::test)]
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
