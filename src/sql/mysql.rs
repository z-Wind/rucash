use std::ops::Deref;

use super::SQLBook;
use crate::SQLError;

#[derive(Debug)]
pub struct Book(SQLBook);

impl Deref for Book {
    type Target = SQLBook;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Book {
    /// Options and flags which can be used to configure a `MySQL` connection.
    /// Described by [`MySQL`](https://dev.mysql.com/doc/connector-j/8.0/en/connector-j-reference-jdbc-url-format.html).
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
    pub async fn new(uri: &str) -> Result<Self, SQLError> {
        let pool = sqlx::any::AnyPoolOptions::new()
            .max_connections(super::MAX_CONNECTIONS)
            .connect(uri)
            .await;

        Ok(Self(SQLBook::new(uri.parse()?, pool?).await))
    }
}
