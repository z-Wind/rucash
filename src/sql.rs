#[cfg(feature = "mysql")]
pub(crate) mod mysql;
#[cfg(feature = "postgres")]
pub(crate) mod postgresql;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;

pub mod wrap;

use super::kind::SQLKind;
use super::model;
use futures::executor::block_on;
use wrap::DataWithPool;

#[derive(Debug)]
pub struct SQLBook {
    kind: SQLKind,
    pool: sqlx::AnyPool,
}

impl SQLBook {
    fn new(kind: SQLKind, pool: sqlx::AnyPool) -> Self {
        Self { kind, pool }
    }

    pub fn accounts(&self) -> Result<Vec<DataWithPool<model::Account>>, sqlx::Error> {
        block_on(async { model::Account::query().fetch_all(&self.pool).await }).map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn account_by_name(
        &self,
        name: &str,
    ) -> Result<Option<DataWithPool<model::Account>>, sqlx::Error> {
        self.accounts_contains_name(name).map(|mut x| x.pop())
    }

    pub fn accounts_contains_name(
        &self,
        name: &str,
    ) -> Result<Vec<DataWithPool<model::Account>>, sqlx::Error> {
        let name = format!("%{}%", name);
        block_on(async {
            model::Account::query_like_name(&name, self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn splits(&self) -> Result<Vec<DataWithPool<model::Split>>, sqlx::Error> {
        block_on(async { model::Split::query().fetch_all(&self.pool).await }).map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn transactions(&self) -> Result<Vec<DataWithPool<model::Transaction>>, sqlx::Error> {
        block_on(async { model::Transaction::query().fetch_all(&self.pool).await }).map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn prices(&self) -> Result<Vec<DataWithPool<model::Price>>, sqlx::Error> {
        block_on(async { model::Price::query().fetch_all(&self.pool).await }).map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn currencies(&self) -> Result<Vec<DataWithPool<model::Commodity>>, sqlx::Error> {
        block_on(async {
            model::Commodity::query_by_namespace("CURRENCY", self.kind)
                .fetch_all(&self.pool)
                .await
        })
        .map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }

    pub fn commodities(&self) -> Result<Vec<DataWithPool<model::Commodity>>, sqlx::Error> {
        block_on(async { model::Commodity::query().fetch_all(&self.pool).await }).map(|v| {
            v.into_iter()
                .map(|x| DataWithPool::new(x, self.kind, self.pool.clone()))
                .collect()
        })
    }
}
