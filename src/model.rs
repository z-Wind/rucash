mod account;
mod commodity;
mod price;
mod split;
mod transaction;

pub use account::Account;
pub use commodity::Commodity;
pub use price::Price;
pub use split::Split;
pub use transaction::Transaction;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql",))]
type TestSchemas<'q, DB, Row, Data, Arguments> =
    sqlx::query::Map<'q, DB, fn(Row) -> Result<Data, sqlx::Error>, Arguments>;

pub trait NullNone {
    #[must_use]
    fn null_none(self) -> Self;
}
