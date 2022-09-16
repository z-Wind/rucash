//! ```rust
//! #[cfg(feature = "sqlite")]
//! {
//!     use rucash::SqliteBook;
//!     let book = SqliteBook::new("sqlite://tests/db/sqlite/complex_sample.gnucash?mode=ro").unwrap();
//!     let accounts = book.accounts();
//! }
//! ```
pub mod model;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
mod kind;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub mod sql;
#[cfg(feature = "xml")]
pub mod xml;

#[cfg(feature = "mysql")]
pub use sql::mysql::MySQLBook;
#[cfg(feature = "postgres")]
pub use sql::postgresql::PostgreSQLBook;
#[cfg(feature = "sqlite")]
pub use sql::sqlite::SqliteBook;
#[cfg(feature = "xml")]
pub use xml::XMLBook;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLAccount = sql::DataWithPool<model::Account>;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLSplit = sql::DataWithPool<model::Split>;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLTransaction = sql::DataWithPool<model::Transaction>;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLPrice = sql::DataWithPool<model::Price>;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLCommodity = sql::DataWithPool<model::Commodity>;

#[cfg(feature = "xml")]
pub type XMLAccount = xml::DataWithPool<model::Account>;
#[cfg(feature = "xml")]
pub type XMLSplit = xml::DataWithPool<model::Split>;
#[cfg(feature = "xml")]
pub type XMLTransaction = xml::DataWithPool<model::Transaction>;
#[cfg(feature = "xml")]
pub type XMLPrice = xml::DataWithPool<model::Price>;
#[cfg(feature = "xml")]
pub type XMLCommodity = xml::DataWithPool<model::Commodity>;
