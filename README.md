<h1 align="center">rucash</h1>
<div align="center">
 <strong>
   The Rust for Gnucash
 </strong>
</div>

<br />

<div align="center">
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
use rucash::prelude::*;
use rucash::SqliteBook;

let book = SqliteBook::new("sqlite://money.gnucash").unwrap();
let accounts = book.accounts();
```

## Install
```toml
# Cargo.toml
[dependencies]
rucash = { version = "0.1", features = [ "sqlite" ] }
```

#### Cargo Feature Flags
-   `sqlite`: Add support for the self-contained [SQLite](https://sqlite.org/) database engine.
-   `postgres`: Add support for the Postgres database server.
-   `mysql`: Add support for the MySQL database server.
-   `xml`: Add support for xml.