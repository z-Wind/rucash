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
xml file should be decompressed to real xml file.
## Example
```rust
let book = rucash::Book::new("sqlite://money.gnucash").unwrap();
let accounts = book.accounts();
```
