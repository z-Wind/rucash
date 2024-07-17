pub(crate) mod account;
pub(crate) mod commodity;
pub(crate) mod price;
pub(crate) mod split;
pub(crate) mod transaction;

use rusqlite::{Connection, OpenFlags};
use std::sync::{Arc, Mutex};

use super::Query;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct SQLiteQuery {
    conn: Arc<Mutex<Connection>>,
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
    pub fn new(uri: &str) -> Result<Self, Error> {
        let conn = Connection::open_with_flags(
            uri,
            OpenFlags::SQLITE_OPEN_READ_ONLY
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let conn = Arc::new(Mutex::new(conn));

        Ok(Self { conn })
    }
}

impl Query for SQLiteQuery {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let uri: &str = &format!(
            "{}/tests/db/sqlite/complex_sample.gnucash",
            env!("CARGO_MANIFEST_DIR")
        );

        println!("work_dir: {:?}", std::env::current_dir());
        SQLiteQuery::new(uri).unwrap();
    }
}
