pub(crate) mod account;
pub(crate) mod commodity;
pub(crate) mod price;
pub(crate) mod split;
pub(crate) mod transaction;

use r2d2::ManageConnection;
use rusqlite::{Connection, OpenFlags};
use tracing::instrument;

use super::Query;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct SQLiteQuery {
    pool: r2d2::Pool<SqliteManager>,
}

impl SQLiteQuery {
    /// Options and flags which can be used to configure a `SQLite` connection.
    /// Described by [`SQLite`](https://www.sqlite.org/uri.html).
    ///
    /// | URI | Description |
    /// | -- | -- |
    /// `file::memory:` | Open an in-memory database. |
    /// `path-to-db/data.db` | Open the file `data.db` |
    /// `file:/path-to-db/data.db` | Open the file `data.db` |
    #[instrument]
    pub fn new(uri: &str) -> Result<Self, Error> {
        tracing::debug!("initializing sqlite connection pool");

        let manager = SqliteManager::new(uri);

        let pool = r2d2::Pool::builder()
            .max_size(5)
            .connection_timeout(std::time::Duration::from_secs(5))
            .build(manager)?;

        tracing::info!("sqlite connection pool established");
        Ok(Self { pool })
    }
}

impl Query for SQLiteQuery {}

#[derive(Debug)]
struct SqliteManager {
    uri: String,
}

impl SqliteManager {
    fn new(uri: impl Into<String>) -> Self {
        Self { uri: uri.into() }
    }
}

impl ManageConnection for SqliteManager {
    type Connection = Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let conn = Connection::open_with_flags(
            &self.uri,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )?;

        // GnuCash 特定優化
        conn.execute_batch(
            "
                PRAGMA query_only = ON;
                PRAGMA cache_size = -8000;
                PRAGMA temp_store = MEMORY;
                PRAGMA mmap_size = 134217728;
                ",
        )?;

        Ok(conn)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.execute_batch("SELECT 1")
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.execute_batch("SELECT 1").is_err()
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    #[test]
    fn test_new() {
        let uri: &str = &format!(
            "{}/tests/db/sqlite/complex_sample.gnucash",
            env!("CARGO_MANIFEST_DIR")
        );

        tracing::debug!("work_dir: {:?}", std::env::current_dir());
        SQLiteQuery::new(uri).unwrap();
    }
}
