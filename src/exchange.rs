use chrono::NaiveDateTime;
#[cfg(not(feature = "decimal"))]
use num_traits::Zero;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;
use tracing::instrument;

use crate::error::Error;
use crate::model::{Commodity, Price};
use crate::query::{CommodityQ, PriceQ, Query};

/// Adjacency list representing the exchange graph.
/// The structure is: Map<From, Map<To, (Rate, Timestamp)>>
type Graph = HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>>;

/// The `Exchange` struct manages currency conversions by maintaining a
/// directed graph of commodity prices and their historical timestamps.
#[derive(Debug, Clone)]
pub(crate) struct Exchange {
    graph: Graph,
}

impl Exchange {
    /// Creates a new `Exchange` instance by fetching data from the provided query provider.
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

    /// Internal logic to build the exchange graph from scratch.
    /// It fetches all prices and commodities to establish conversion edges.
    #[instrument(skip(query))]
    async fn new_graph<Q>(query: Arc<Q>) -> Result<Graph, Error>
    where
        Q: Query,
    {
        /// Helper function to update or insert an edge in the graph.
        /// It only updates the edge if the new price entry has a more recent timestamp.
        fn upsert_edge(
            graph: &mut Graph,
            from: &str,
            to: &str,
            rate: crate::Num,
            date: NaiveDateTime,
        ) {
            graph
                .entry(from.to_string())
                .or_default()
                .entry(to.to_string())
                .and_modify(|e| {
                    if e.1 < date {
                        tracing::debug!(
                            from = from,
                            to = to,
                            old_date = %e.1,
                            new_date = %date,
                            old_rate = ?e.0,
                            new_rate = ?rate,
                            "updating edge to newer entry"
                        );
                        *e = (rate, date);
                    }
                })
                .or_insert((rate, date));
        }

        tracing::debug!("building exchange graph from prices and commodities");

        // Fetch all prices and convert them into the internal model
        let prices: Vec<Price<Q>> = PriceQ::all(&*query)
            .await
            .inspect_err(|e| tracing::error!("failed to fetch prices: {e}"))?
            .into_iter()
            .map(|x| Price::from_with_query(&x, query.clone()))
            .collect();
        tracing::debug!(price_count = prices.len(), "prices loaded");

        // Create a lookup map for commodity GUIDs to their mnemonics (e.g., "USD", "BTC")
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

        let mut graph: Graph = HashMap::new();

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

            // Zero value prices are invalid for exchange calculations
            if p.value.is_zero() {
                tracing::warn!(
                    datetime = %p.datetime,
                    commodity = commodity,
                    currency = currency,
                    "ignoring price with zero value in exchange graph"
                );
                continue;
            }

            // Insert forward edge: commodity -> currency
            upsert_edge(&mut graph, commodity, currency, p.value, p.datetime);

            // Insert reverse edge: currency -> commodity (reciprocal rate)
            upsert_edge(
                &mut graph,
                currency,
                commodity,
                num_traits::one::<crate::Num>() / p.value,
                p.datetime,
            );
        }

        tracing::info!(
            graph_nodes = graph.len(),
            total_edges = graph.values().map(HashMap::len).sum::<usize>(),
            "exchange graph built successfully"
        );
        Ok(graph)
    }

    /// Calculates the exchange rate from one commodity to another.
    ///
    /// It uses a priority-search (Dijkstra-like) approach where it prioritizes
    /// paths that contain the "freshest" data (most recent `oldest_edge_date`).
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

        // Identity case: converting a commodity to itself
        if commodity == currency {
            tracing::debug!("same commodity and currency, returning 1.0");
            return Some(num_traits::one());
        }

        tracing::debug!("searching for most reliable exchange path");

        let mut visited: HashSet<String> = HashSet::new();
        let mut heap: BinaryHeap<ExchangePath> = BinaryHeap::new();

        // Seed the heap with the starting commodity
        heap.push(ExchangePath {
            node: commodity.clone(),
            rate: num_traits::one(),
            oldest_edge_date: chrono::NaiveDateTime::MAX,
            hop_count: 0,
        });

        while let Some(ExchangePath {
            node,
            rate,
            oldest_edge_date,
            hop_count,
        }) = heap.pop()
        {
            // Goal check
            if &node == currency {
                tracing::info!(
                    ?rate,
                    oldest_edge_date = %oldest_edge_date,
                    hop_count = hop_count,
                    "found most reliable exchange path"
                );
                return Some(rate);
            }

            // Skip if already processed
            if !visited.insert(node.clone()) {
                continue;
            }

            // Explore neighbors
            if let Some(neighbors) = self.graph.get(&node) {
                for (next, (edge_rate, edge_date)) in neighbors {
                    if !visited.contains(next) {
                        heap.push(ExchangePath {
                            node: next.clone(),
                            rate: rate * edge_rate,
                            oldest_edge_date: oldest_edge_date.min(*edge_date),
                            hop_count: hop_count + 1,
                        });
                    }
                }
            }
        }

        tracing::warn!("no exchange path found between commodities");
        None
    }

    /// Rebuilds the exchange graph with the latest data.
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

/// Represents a potential conversion path in the priority queue.
#[derive(Debug, Clone)]
struct ExchangePath {
    node: String,
    rate: crate::Num,
    /// The bottleneck date: the timestamp of the oldest price used in this path.
    oldest_edge_date: chrono::NaiveDateTime,
    /// Number of conversions (steps) in the path.
    hop_count: usize,
}

/// Custom ordering for the priority queue:
/// 1. Prefer paths where the `oldest_edge_date` is later (more recent data).
/// 2. If dates are equal, prefer paths with fewer hops (shorter distance).
impl Ord for ExchangePath {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.oldest_edge_date.cmp(&other.oldest_edge_date) {
            Ordering::Equal => other.hop_count.cmp(&self.hop_count),
            ord => ord,
        }
    }
}

impl PartialOrd for ExchangePath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ExchangePath {}

impl PartialEq for ExchangePath {
    fn eq(&self, other: &Self) -> bool {
        self.oldest_edge_date == other.oldest_edge_date
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
