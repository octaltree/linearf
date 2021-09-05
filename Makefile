MAKEFLAGS += --no-print-directory

.PHONY: clean
clean:
	cd core && make clean
	cd bridge && make clean
	cd tests && make clean


# Development
.PHONY: dev
dev:
	@cd core && make dev
	@cd bridge && make dev
	@cd lua && make dev
	@cd tests && make dev
	@cd bridge && make dev

.PHONY: vl
vl:
	@cd lua && make dev
	@cd tests && make dev

.PHONY: d
d:
	@watchexec -c 'make dev'


# vim: foldmethod=marker
