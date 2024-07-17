all: test build
build:	
	cargo build
test:
	cargo test --features sqlite,postgresql,mysql,xml,sqlitefaster
	cargo test --features sqlite
	cargo test --features sqlitefaster
	cargo test --features postgresql
	cargo test --features mysql
	cargo test --features xml
	cargo test --features sqlite,postgresql,mysql,xml,sqlitefaster,decimal
	cargo test --features sqlite,decimal
	cargo test --features sqlitefaster,decimal
	cargo test --features postgresql,decimal
	cargo test --features mysql,decimal
	cargo test --features xml,decimal
clean:
	cargo clean
bench:
	cargo bench --features sqlite,xml,sqlitefaster
check:
	cargo check --features sqlite,postgresql,mysql,xml,sqlitefaster --all-targets
	cargo clippy --features sqlite,postgresql,mysql,xml,sqlitefaster --all-targets
	cargo check --features sqlite,postgresql,mysql,xml,sqlitefaster,decimal --all-targets
	cargo clippy --features sqlite,postgresql,mysql,xml,sqlitefaster,decimal --all-targets
checkschema:
	export DATABASE_URL=sqlite://tests/db/sqlite/complex_sample.gnucash?mode=ro
	cargo check --features sqlite,schema --all-targets
	export DATABASE_URL=mysql://user:secret@localhost/complex_sample.gnucash
	cargo check --features mysql,schema --all-targets
	export DATABASE_URL=postgresql://user:secret@localhost:5432/complex_sample.gnucash
	cargo check --features postgresql,schema --all-targets
publish:
	cargo publish --all-features
backup:
# Using Gnucash to convert test files into MySQL and PostgreSQL formats, 
# then dump them out, and restore them in the testing environment.
	pg_dump -U user -f "tests/db/postgresql/complex_sample.gnucash.sql" complex_sample.gnucash
	mysqldump -h localhost -u user -p complex_sample.gnucash > "tests/db/mysql/complex_sample.gnucash.sql"