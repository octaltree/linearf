MAKEFLAGS += --no-print-directory

.PHONY: dev
dev:
	@cd lua && make dev
	@cd model && make dev
	@cd tests && make dev

.PHONY: clean
clean:
	cd tests && make clean

.PHONY: doc
doc:
	cd doc/tpl && cargo run

.PHONY: d
d:
	@watchexec -i model/registry -c 'make dev'
