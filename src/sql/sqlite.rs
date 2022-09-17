use super::SQLBook;
use futures::executor::block_on;
use std::ops::Deref;

pub struct SqliteBook(SQLBook);

impl Deref for SqliteBook {
    type Target = SQLBook;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SqliteBook {
    /// Options and flags which can be used to configure a SQLite connection.
    /// Described by [SQLite](https://www.sqlite.org/uri.html).
    ///
    /// | URI | Description |
    /// | -- | -- |
    /// `sqlite::memory:` | Open an in-memory database. |
    /// `sqlite:data.db` | Open the file `data.db` in the current directory. |
    /// `sqlite://data.db` | Open the file `data.db` in the current directory. |
    /// `sqlite:///data.db` | Open the file `data.db` from the root (`/`) directory. |
    /// `sqlite://data.db?mode=ro` | Open the file `data.db` for read-only access. |
    pub fn new(uri: &str) -> Result<Self, sqlx::Error> {
        let pool = block_on(async {
            sqlx::any::AnyPoolOptions::new()
                .max_connections(5)
                .connect(uri)
                .await
        });
        Ok(Self(SQLBook::new(uri.parse()?, pool?)))
    }
}
