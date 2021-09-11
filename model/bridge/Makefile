.PHONY: clean
clean:
	rm -rf target

.PHONY: dev
dev: format lint test doc
	@cargo build --features=mlua/luajit --quiet

.PHONY: d
d:
	@cargo watch -e -c -s 'make dev'

.PHONY: format
format:
	@rustup run nightly cargo fmt --quiet

.PHONY: lint
lint:
	@cargo clippy --all-targets --features=mlua/luajit --quiet

.PHONY: test
test:
	@cargo test --all-targets --features=mlua/luajit --quiet

.PHONY: doc
doc:
	@cargo doc --features=mlua/luajit --quiet
