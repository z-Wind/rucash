pub(crate) mod account;
pub(crate) mod commodity;
pub(crate) mod price;
pub(crate) mod split;
pub(crate) mod transaction;

use super::Query;
use crate::error::Error;

const MAX_CONNECTIONS: u32 = 10;

#[derive(Debug, Clone)]
pub struct SQLiteQuery {
    pool: sqlx::SqlitePool,
}

impl SQLiteQuery {
    /// Options and flags which can be used to configure a `SQLite` connection.
    /// Described by [`SQLite`](https://www.sqlite.org/uri.html).
    ///
    /// | URI | Description |
    /// | -- | -- |
    /// `sqlite::memory:` | Open an in-memory database. |
    /// `sqlite:data.db` | Open the file `data.db` in the current directory. |
    /// `sqlite://data.db` | Open the file `data.db` in the current directory. |
    /// `sqlite:///data.db` | Open the file `data.db` from the root (`/`) directory. |
    /// `sqlite://data.db?mode=ro` | Open the file `data.db` for read-only access. |
    pub async fn new(uri: &str) -> Result<Self, Error> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect(uri)
            .await?;

        Ok(Self { pool })
    }
}

impl Query for SQLiteQuery {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new() {
        let uri: &str = &format!(
            "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
            env!("CARGO_MANIFEST_DIR")
        );

        println!("work_dir: {:?}", std::env::current_dir());
        SQLiteQuery::new(&format!("{uri}?mode=ro")).await.unwrap();
    }
}
