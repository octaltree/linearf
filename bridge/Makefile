.PHONY: clean
clean:
	rm -rf target

.PHONY: dev
dev: format lint test doc
	@cargo build --features=luajit,vendored --quiet

.PHONY: d
d:
	@cargo watch -c -s 'make dev'

.PHONY: format
format:
	@cargo fmt --quiet

.PHONY: lint
lint:
	@cargo clippy --all-targets --features=luajit --quiet

.PHONY: test
test:
	@cargo test --all-targets --features=luajit --quiet

.PHONY: doc
doc:
	@cargo doc --features=luajit --quiet
