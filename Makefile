MAKEFLAGS += --no-print-directory

.PHONY: clean
clean:
	cd model && make clean
	cd tests && make clean


# Development
.PHONY: dev
dev:
	@cd model && make dev
	@cd lua && make dev
	@cd tests && make dev

.PHONY: vl
vl:
	@cd lua && make dev
	@cd tests && make dev

.PHONY: d
d:
	@watchexec -i model/registrar/registrar -c 'make dev'


# vim: foldmethod=marker
