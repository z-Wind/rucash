//! ```rust
//! #[cfg(feature = "sqlite")]
//! {
//!     use rucash::SqliteBook;
//!
//!     #[tokio::main]
//!     async fn main() {
//!         let book = SqliteBook::new("sqlite://tests/db/sqlite/complex_sample.gnucash?mode=ro").await.unwrap();
//!         let accounts = book.accounts();
//!     }
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(
    clippy::pedantic,
    missing_copy_implementations,
    missing_debug_implementations,
    //missing_docs,
    rustdoc::broken_intra_doc_links,
    trivial_numeric_casts,
    unused_allocation
)]
#![allow(clippy::missing_errors_doc)]

pub mod model;

#[cfg(not(feature = "decimal"))]
pub type Num = f64;
#[cfg(feature = "decimal")]
pub type Num = rust_decimal::Decimal;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
mod kind;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub mod sql;
#[cfg(feature = "xml")]
pub mod xml;

#[cfg(feature = "mysql")]
pub use sql::mysql::Book as MySQLBook;
#[cfg(feature = "postgres")]
pub use sql::postgresql::Book as PostgreSQLBook;
#[cfg(feature = "sqlite")]
pub use sql::sqlite::Book as SqliteBook;
#[cfg(feature = "xml")]
pub use xml::Book as XMLBook;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLAccount = sql::wrap::DataWithPool<model::Account>;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLSplit = sql::wrap::DataWithPool<model::Split>;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLTransaction = sql::wrap::DataWithPool<model::Transaction>;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLPrice = sql::wrap::DataWithPool<model::Price>;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLCommodity = sql::wrap::DataWithPool<model::Commodity>;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub type SQLError = sql::error::Error;

#[cfg(feature = "xml")]
pub type XMLAccount = xml::wrap::DataWithPool<model::Account>;
#[cfg(feature = "xml")]
pub type XMLSplit = xml::wrap::DataWithPool<model::Split>;
#[cfg(feature = "xml")]
pub type XMLTransaction = xml::wrap::DataWithPool<model::Transaction>;
#[cfg(feature = "xml")]
pub type XMLPrice = xml::wrap::DataWithPool<model::Price>;
#[cfg(feature = "xml")]
pub type XMLCommodity = xml::wrap::DataWithPool<model::Commodity>;
#[cfg(feature = "xml")]
pub type XMLError = xml::error::Error;
