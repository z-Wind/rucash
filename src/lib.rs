//! ```rust
//! #[cfg(feature = "sqlite")]
//! {
//!     use rucash::prelude::*;
//!     use rucash::SqliteBook;
//!     let book = SqliteBook::new("sqlite://tests/db/sqlite/complex_sample.gnucash?mode=ro").unwrap();
//!     let accounts = book.accounts();
//! }
//! ```
pub mod model;
pub mod template;

#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgresql;
#[cfg(feature = "sqlite")]
pub mod sqlite;
#[cfg(feature = "xml")]
pub mod xml;

#[cfg(feature = "mysql")]
pub use mysql::MySQLBook;
#[cfg(feature = "postgres")]
pub use postgresql::PostgreSQLBook;
#[cfg(feature = "sqlite")]
pub use sqlite::SqliteBook;
#[cfg(feature = "xml")]
pub use xml::XMLBook;

/// A convenience module appropriate for glob imports (`use rust::prelude::*;`).
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::template::AccountT;
    #[doc(no_inline)]
    pub use crate::template::BookT;
    #[doc(no_inline)]
    pub use crate::template::CommodityT;
    #[doc(no_inline)]
    pub use crate::template::PriceT;
    #[doc(no_inline)]
    pub use crate::template::SplitT;
    #[doc(no_inline)]
    pub use crate::template::TransactionT;
}
