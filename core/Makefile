.PHONY: release dev d format lint test doc

release:
	cargo build --release
	cargo doc

dev: format lint test doc

d:
	cargo watch -c -s 'make dev'

format:
	cargo fmt

lint:
	cargo clippy --all-targets

test:
	cargo test --all-targets

doc:
	cargo doc
