pub(crate) mod account;
pub(crate) mod commodity;
pub(crate) mod price;
pub(crate) mod split;
pub(crate) mod transaction;

use super::Query;
use crate::error::Error;

const MAX_CONNECTIONS: u32 = 10;

#[derive(Debug, Clone)]
pub struct MySQLQuery {
    pool: sqlx::MySqlPool,
}

impl MySQLQuery {
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
    pub async fn new(uri: &str) -> Result<Self, Error> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect(uri)
            .await?;

        Ok(Self { pool })
    }
}

impl Query for MySQLQuery {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new() {
        let uri: &str = "mysql://user:secret@localhost/complex_sample.gnucash";

        println!("work_dir: {:?}", std::env::current_dir());
        MySQLQuery::new(&format!("{uri}?mode=ro")).await.unwrap();
    }
}
