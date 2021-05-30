all: test build
build:	
	cargo build
test:
	cargo test --all-features
	cargo test --features sqlite
	cargo test --features postgres
	cargo test --features mysql
	cargo test --features xml
clean:
	cargo clean
bench:
	cargo bench --features sqlite
