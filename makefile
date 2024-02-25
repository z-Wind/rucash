all: test build
build:	
	cargo build
test:
	cargo test --features sqlite,postgresql,mysql,xml
	cargo test --features sqlite
	cargo test --features postgresql
	cargo test --features mysql
	cargo test --features xml
	cargo test --features sqlite,postgresql,mysql,xml,decimal
	cargo test --features sqlite,decimal
	cargo test --features postgresql,decimal
	cargo test --features mysql,decimal
	cargo test --features xml,decimal
clean:
	cargo clean
bench:
	cargo bench --features sqlite,xml
check:
	cargo check --features sqlite,postgresql,mysql,xml --all-targets
	cargo clippy --features sqlite,postgresql,mysql,xml --all-targets
	cargo check --features sqlite,postgresql,mysql,xml,decimal --all-targets
	cargo clippy --features sqlite,postgresql,mysql,xml,decimal --all-targets
checkschema:
	DATABASE_URL=sqlite://tests/db/sqlite/complex_sample.gnucash?mode=ro;
	cargo check --features sqlite,schema
	DATABASE_URL=mysql://user:secret@localhost/complex_sample.gnucash;
	cargo check --features mysql,schema
	DATABASE_URL=postgresql://user:secret@localhost:5432/complex_sample.gnucash;
	cargo check --features postgresql,schema
publish:
	cargo publish --all-features
prepare:
	cargo sqlx prepare --database-url sqlite://tests/db/sqlite/complex_sample.gnucash?mode=ro -- --tests --features sqlite,schema
	cargo sqlx prepare --database-url mysql://user:secret@localhost/complex_sample.gnucash -- --tests --features mysql,schema
	cargo sqlx prepare --database-url postgresql://user:secret@localhost:5432/complex_sample.gnucash -- --tests --features postgres,schema
