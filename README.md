<h1 align="center">rucash</h1>
<div align="center">
 <strong>
   The Rust for Gnucash
 </strong>
</div>

<br />

<div align="center">
  <!-- CI -->
  <a href="https://github.com/z-Wind/rucash/actions/workflows/ci.yml">
    <img src="https://github.com/z-Wind/rucash/actions/workflows/ci.yml/badge.svg?style=flat-square"
    alt="CI Result" />
  </a>
  <!-- Version -->
  <a href="https://crates.io/crates/rucash">
    <img src="https://img.shields.io/crates/v/rucash.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Docs -->
  <a href="https://docs.rs/rucash">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/rucash">
    <img src="https://img.shields.io/crates/d/rucash.svg?style=flat-square"
      alt="Download" />
  </a>
</div>

<br/>

rucash provides a simple interface to GnuCash files stored in xml and SQL (sqlite3, PostgreSQL and MySQL).
## Example
```rust
use rucash::{Book, SQLiteQuery};

#[tokio::main]
async fn main() {
    let query = SQLiteQuery::new("sqlite://tests/db/sqlite/complex_sample.gnucash?mode=ro").await.unwrap();
    let book = Book::new(query).await.unwrap();
    let accounts = book.accounts();
}
```

## Install
```toml
# Cargo.toml
[dependencies]
rucash = { version = "0.4", features = [ "sqlite", "decimal" ] }
```

#### Cargo Feature Flags
-   `sqlite`: Add support for the self-contained [SQLite](https://sqlite.org/) database engine.
-   `postgresql`: Add support for the Postgres database server.
-   `mysql`: Add support for the MySQL database server.
-   `xml`: Add support for xml.
-   `decimal`: Add support for Decimal.