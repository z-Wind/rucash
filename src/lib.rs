#[cfg(any(
    feature = "sqlite",
    feature = "postgres",
    feature = "mysql",
    feature = "xml"
))]
pub mod model;
#[cfg(any(
    feature = "sqlite",
    feature = "postgres",
    feature = "mysql",
    feature = "xml"
))]
pub mod template;

#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgresql;
#[cfg(feature = "sqlite")]
pub mod sqlite;
#[cfg(feature = "xml")]
pub mod xml;
