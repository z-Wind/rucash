#[cfg(feature = "mysql")]
pub(crate) mod mysql;
#[cfg(feature = "postgres")]
pub(crate) mod postgresql;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;

mod exchange;
pub mod wrap;

use std::sync::Arc;
use tokio::sync::RwLock;

use super::kind::SQLKind;
use super::model;
use exchange::Exchange;
use wrap::DataWithPool;

#[derive(Debug)]
pub struct SQLBook {
    kind: SQLKind,
    pool: sqlx::AnyPool,
    exchange_graph: Option<Arc<RwLock<Exchange>>>,
}

impl SQLBook {
    async fn new(kind: SQLKind, pool: sqlx::AnyPool) -> Self {
        let exchange_graph = Exchange::new(kind, pool.clone())
            .await
            .map(|x| Arc::new(RwLock::new(x)))
            .ok();
        Self {
            kind,
            pool,
            exchange_graph,
        }
    }

    pub async fn accounts(&self) -> Result<Vec<DataWithPool<model::Account>>, sqlx::Error> {
        model::Account::query()
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
    }

    pub async fn account_by_name(
        &self,
        name: &str,
    ) -> Result<Option<DataWithPool<model::Account>>, sqlx::Error> {
        self.accounts_contains_name(name).await.map(|mut x| x.pop())
    }

    pub async fn accounts_contains_name(
        &self,
        name: &str,
    ) -> Result<Vec<DataWithPool<model::Account>>, sqlx::Error> {
        let name = format!("%{}%", name);

        model::Account::query_like_name(&name, self.kind)
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
    }

    pub async fn splits(&self) -> Result<Vec<DataWithPool<model::Split>>, sqlx::Error> {
        model::Split::query().fetch_all(&self.pool).await.map(|v| {
            v.into_iter()
                .map(|x| {
                    DataWithPool::new(x, self.kind, self.pool.clone(), self.exchange_graph.clone())
                })
                .collect()
        })
    }

    pub async fn transactions(&self) -> Result<Vec<DataWithPool<model::Transaction>>, sqlx::Error> {
        model::Transaction::query()
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
    }

    pub async fn prices(&self) -> Result<Vec<DataWithPool<model::Price>>, sqlx::Error> {
        model::Price::query().fetch_all(&self.pool).await.map(|v| {
            v.into_iter()
                .map(|x| {
                    DataWithPool::new(x, self.kind, self.pool.clone(), self.exchange_graph.clone())
                })
                .collect()
        })
    }

    pub async fn currencies(&self) -> Result<Vec<DataWithPool<model::Commodity>>, sqlx::Error> {
        model::Commodity::query_by_namespace("CURRENCY", self.kind)
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
    }

    pub async fn commodities(&self) -> Result<Vec<DataWithPool<model::Commodity>>, sqlx::Error> {
        model::Commodity::query()
            .fetch_all(&self.pool)
            .await
            .map(|v| {
                v.into_iter()
                    .map(|x| {
                        DataWithPool::new(
                            x,
                            self.kind,
                            self.pool.clone(),
                            self.exchange_graph.clone(),
                        )
                    })
                    .collect()
            })
    }
}
