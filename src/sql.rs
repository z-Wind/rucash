#[cfg(feature = "mysql")]
pub(crate) mod mysql;
#[cfg(feature = "postgres")]
pub(crate) mod postgresql;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;

pub mod error;
mod exchange;
pub mod wrap;

use std::sync::Arc;
use tokio::sync::RwLock;

use super::kind::SQLKind;
use super::model;
use crate::SQLError;
use exchange::Exchange;
use wrap::DataWithPool;

const MAX_CONNECTIONS: u32 = 10;

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

    pub async fn accounts(&self) -> Result<Vec<DataWithPool<model::Account>>, SQLError> {
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
            .map_err(std::convert::Into::into)
    }

    pub async fn account_by_name(
        &self,
        name: &str,
    ) -> Result<Option<DataWithPool<model::Account>>, SQLError> {
        self.accounts_contains_name(name).await.map(|mut x| x.pop())
    }

    pub async fn accounts_contains_name(
        &self,
        name: &str,
    ) -> Result<Vec<DataWithPool<model::Account>>, SQLError> {
        let name = format!("%{name}%");

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
            .map_err(std::convert::Into::into)
    }

    pub async fn splits(&self) -> Result<Vec<DataWithPool<model::Split>>, SQLError> {
        model::Split::query()
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
            .map_err(std::convert::Into::into)
    }

    pub async fn transactions(&self) -> Result<Vec<DataWithPool<model::Transaction>>, SQLError> {
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
            .map_err(std::convert::Into::into)
    }

    pub async fn prices(&self) -> Result<Vec<DataWithPool<model::Price>>, SQLError> {
        model::Price::query()
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
            .map_err(std::convert::Into::into)
    }

    pub async fn currencies(&self) -> Result<Vec<DataWithPool<model::Commodity>>, SQLError> {
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
            .map_err(std::convert::Into::into)
    }

    pub async fn commodities(&self) -> Result<Vec<DataWithPool<model::Commodity>>, SQLError> {
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
            .map_err(std::convert::Into::into)
    }
}
