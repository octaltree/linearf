.PHONY: all dev d format lint test doc

all: format lint test doc
	cargo build --release

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
