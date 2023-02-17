use super::SQLBook;
use std::ops::Deref;

#[derive(Debug)]
pub struct PostgreSQLBook(SQLBook);

impl Deref for PostgreSQLBook {
    type Target = SQLBook;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PostgreSQLBook {
    /// Options and flags which can be used to configure a `PostgreSQL` connection.
    /// Described by [libpq](https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING).
    ///
    /// The general form for a connection URI is:
    ///
    /// ```text
    /// postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
    /// ```
    ///
    /// ## Parameters
    ///
    /// |Parameter|Default|Description|
    /// |---------|-------|-----------|
    /// | `sslmode` | `prefer` | Determines whether or with what priority a secure SSL TCP/IP connection will be negotiated |
    /// | `sslrootcert` | `None` | Sets the name of a file containing a list of trusted SSL Certificate Authorities. |
    /// | `statement-cache-capacity` | `100` | The maximum number of prepared statements stored in the cache. Set to `0` to disable. |
    /// | `host` | `None` | Path to the directory containing a `PostgreSQL` unix domain socket, which will be used instead of TCP if set. |
    /// | `hostaddr` | `None` | Same as `host`, but only accepts IP addresses. |
    /// | `application-name` | `None` | The name will be displayed in the `pg_stat_activity` view and included in CSV log entries. |
    /// | `user` | result of `whoami` |  `PostgreSQL` user name to connect as. |
    /// | `password` | `None` | Password to be used if the server demands password authentication. |
    /// | `port` | `5432` | Port number to connect to at the server host, or socket file name extension for Unix-domain connections. |
    /// | `dbname` | `None` | The database name. |
    ///
    /// The URI scheme designator can be either `postgresql://` or `postgres://`.
    /// Each of the URI parts is optional.
    ///
    /// ```text
    /// postgresql://
    /// postgresql://localhost
    /// postgresql://localhost:5433
    /// postgresql://localhost/mydb
    /// postgresql://user@localhost
    /// postgresql://user:secret@localhost
    /// postgresql://localhost?dbname=mydb&user=postgres&password=postgres
    /// ```
    pub async fn new(uri: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::any::AnyPoolOptions::new()
            .max_connections(super::MAX_CONNECTIONS)
            .connect(uri)
            .await;

        Ok(Self(SQLBook::new(uri.parse()?, pool?).await))
    }
}
