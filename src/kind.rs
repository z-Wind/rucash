use std::str::FromStr;

use crate::SQLError;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum SQLKind {
    Postgres,
    MySql,
    Sqlite,
    Mssql,
}

impl FromStr for SQLKind {
    type Err = SQLError;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        match url {
            #[cfg(feature = "postgres")]
            _ if url.starts_with("postgres:") || url.starts_with("postgresql:") => {
                Ok(SQLKind::Postgres)
            }

            #[cfg(not(feature = "postgres"))]
            _ if url.starts_with("postgres:") || url.starts_with("postgresql:") => {
                Err(sqlx::Error::Configuration("database URL has the scheme of a PostgreSQL database but the `postgres` feature is not enabled".into())).map_err(std::convert::Into::into)
            }

            #[cfg(feature = "mysql")]
            _ if url.starts_with("mysql:") || url.starts_with("mariadb:") => {
                Ok(SQLKind::MySql)
            }

            #[cfg(not(feature = "mysql"))]
            _ if url.starts_with("mysql:") || url.starts_with("mariadb:") => {
                Err(sqlx::Error::Configuration("database URL has the scheme of a MySQL database but the `mysql` feature is not enabled".into())).map_err(std::convert::Into::into)
            }

            #[cfg(feature = "sqlite")]
            _ if url.starts_with("sqlite:") => {
                Ok(SQLKind::Sqlite)
            }

            #[cfg(not(feature = "sqlite"))]
            _ if url.starts_with("sqlite:") => {
                Err(sqlx::Error::Configuration("database URL has the scheme of a SQLite database but the `sqlite` feature is not enabled".into())).map_err(std::convert::Into::into)
            }

            #[cfg(feature = "mssql")]
            _ if url.starts_with("mssql:") || url.starts_with("sqlserver:") => {
                Ok(SQLKind::Mssql)
            }

            #[cfg(not(feature = "mssql"))]
            _ if url.starts_with("mssql:") || url.starts_with("sqlserver:") => {
                Err(sqlx::Error::Configuration("database URL has the scheme of a MSSQL database but the `mssql` feature is not enabled".into())).map_err(std::convert::Into::into)
            }

            _ => Err(sqlx::Error::Configuration(format!("unrecognized database url: {url:?}").into())).map_err(std::convert::Into::into)
        }
    }
}
