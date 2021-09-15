MAKEFLAGS += --no-print-directory

.PHONY: dev
dev:
	@cd lua && make dev
	@cd model && make dev
	@cd tests && make dev

.PHONY: clean
clean:
	cd tests && make clean

.PHONY: d
d:
	@watchexec -i model/registrar -c 'make dev'
