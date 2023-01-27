use super::wrap::DataWithPool;
use crate::kind::SQLKind;
use crate::model;
use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub(crate) struct Exchange {
    kind: SQLKind,
    pool: sqlx::AnyPool,
    graph: HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>>,
}

impl Exchange {
    async fn new_graph(
        kind: SQLKind,
        pool: sqlx::AnyPool,
    ) -> Result<HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>>, sqlx::Error> {
        let prices: Vec<DataWithPool<model::Price>> =
            model::Price::query().fetch_all(&pool).await.map(|v| {
                v.into_iter()
                    .map(|x| DataWithPool::new(x, kind, pool.clone(), None))
                    .collect()
            })?;

        let mut graph: HashMap<String, HashMap<String, (crate::Num, NaiveDateTime)>> =
            HashMap::new();
        for p in prices {
            let commodity = &p.commodity().await?.mnemonic;
            let currency = &p.currency().await?.mnemonic;

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

        Ok(graph)
    }
    pub(crate) async fn new(kind: SQLKind, pool: sqlx::AnyPool) -> Result<Self, sqlx::Error> {
        Ok(Self {
            graph: Self::new_graph(kind, pool.clone()).await?,
            kind,
            pool,
        })
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

    pub(crate) async fn update(&mut self) -> Result<(), sqlx::Error> {
        self.graph = Self::new_graph(self.kind, self.pool.clone()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        //type DB = sqlx::Sqlite;

        async fn setup() -> crate::SqliteBook {
            let uri: &str = &format!(
                "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );
            crate::SqliteBook::new(uri)
                .await
                .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"))
        }

        #[tokio::test]
        async fn test_exchange() {
            let book = setup().await;
            let mut exchange = Exchange::new(book.kind, book.pool.clone()).await.unwrap();
            exchange.update().await.expect("ok");

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

        const URI: &str = "postgresql://user:secret@localhost:5432/complex_sample.gnucash";
        //type DB = sqlx::Postgres;

        async fn setup(uri: &str) -> crate::PostgreSQLBook {
            crate::PostgreSQLBook::new(&uri)
                .await
                .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"))
        }

        #[tokio::test]
        async fn test_exchange() {
            let book = setup().await;
            let mut exchange = Exchange::new(book.kind, book.pool.clone()).await.unwrap();
            exchange.update().await.expect("ok");

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

        //type DB = sqlx::MySql;

        async fn setup() -> crate::MySQLBook {
            let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";
            crate::MySQLBook::new(uri)
                .await
                .unwrap_or_else(|e| panic!("{e} uri:{uri:?}"))
        }

        #[tokio::test]
        async fn test_exchange() {
            let book = setup().await;
            let mut exchange = Exchange::new(book.kind, book.pool.clone()).await.unwrap();
            exchange.update().await.expect("ok");

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
