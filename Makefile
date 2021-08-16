MAKEFLAGS += --no-print-directory

.PHONY: clean
clean:
	rm -rf tools
	cd core && make clean
	cd bridge && make clean


# Development
.PHONY: dev
dev: vim-lint
	@cd lua && make dev
	@cd bridge && make dev
	@cd core && make dev

.PHONY: d
d:
	@watchexec -c 'make dev'


.PHONY: vim-lint
vim-lint: tools/py/bin/vint
	@./tools/py/bin/vint autoload


## Prepare tools {{{
prepare: tools/py/bin/vint

tools/py/bin/vint: tools/py/bin
	cd tools && ./py/bin/pip install vim-vint

tools/py/bin: tools
	cd tools && python -m venv py

tools:
	mkdir -p $@
# }}}

# vim: foldmethod=marker
