all: test build
build:	
	cargo build
test:
	cargo test --features sqlite,postgres,mysql,xml
	cargo test --features sqlite
	cargo test --features postgres
	cargo test --features mysql
	cargo test --features xml
	cargo test --all-features
	cargo test --features sqlite,decimal
	cargo test --features postgres,decimal
	cargo test --features mysql,decimal
	cargo test --features xml,decimal
clean:
	cargo clean
bench:
	cargo bench --features sqlite,xml
check:
	cargo check --all-features --all-targets
	cargo check --features sqlite,postgres,mysql,xml --all-targets
	cargo clippy --all-features --all-targets
	cargo clippy --features sqlite,postgres,mysql,xml --all-targets
publish:
	cargo publish --all-features
