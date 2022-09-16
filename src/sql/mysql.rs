use super::SQLBook;
use futures::executor::block_on;
use std::ops::Deref;

pub struct MySQLBook(SQLBook);

impl Deref for MySQLBook {
    type Target = SQLBook;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MySQLBook {
    /// Options and flags which can be used to configure a MySQL connection.
    /// Described by [MySQL](https://dev.mysql.com/doc/connector-j/8.0/en/connector-j-reference-jdbc-url-format.html).
    ///
    /// The generic format of the connection URL:
    ///
    /// ```text
    /// mysql://[host][/database][?properties]
    /// ```
    ///
    /// ## Properties
    ///
    /// |Parameter|Default|Description|
    /// |---------|-------|-----------|
    /// | `ssl-mode` | `PREFERRED` | Determines whether or with what priority a secure SSL TCP/IP connection will be negotiated. |
    /// | `ssl-ca` | `None` | Sets the name of a file containing a list of trusted SSL Certificate Authorities. |
    /// | `statement-cache-capacity` | `100` | The maximum number of prepared statements stored in the cache. Set to `0` to disable. |
    /// | `socket` | `None` | Path to the unix domain socket, which will be used instead of TCP if set. |
    ///
    /// ```text
    /// mysql://root:password@localhost/db
    /// ```
    pub fn new(uri: &str) -> Result<Self, sqlx::Error> {
        let pool = block_on(async {
            sqlx::any::AnyPoolOptions::new()
                .max_connections(5)
                .connect(uri)
                .await
        });
        let pool = pool?.clone();
        Ok(Self(SQLBook::new(uri.parse()?, pool)))
    }
}
