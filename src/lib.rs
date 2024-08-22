//! ```rust
//! #[cfg(feature = "sqlite")]
//! {
//!     use rucash::{Book, SQLiteQuery};
//!
//!     #[tokio::main]
//!     async fn main() {
//!         let query = SQLiteQuery::new("tests/db/sqlite/complex_sample.gnucash").unwrap();
//!         let book = Book::new(query).await.unwrap();
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

mod book;
mod error;
mod exchange;
mod query;

pub mod model;

#[cfg(not(feature = "decimal"))]
pub type Num = f64;
#[cfg(feature = "decimal")]
pub type Num = rust_decimal::Decimal;

pub use book::Book;
pub use error::Error;
#[cfg(feature = "mysql")]
pub use query::mysql::MySQLQuery;
#[cfg(feature = "postgresql")]
pub use query::postgresql::PostgreSQLQuery;
#[cfg(feature = "sqlite")]
pub use query::sqlite::SQLiteQuery;
#[cfg(feature = "xml")]
pub use query::xml::XMLQuery;
pub use query::Query;
