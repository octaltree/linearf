PATH := ./tools/ve/bin:$(PATH)

.PHONY: dev
dev: core lua vim

.PHONY: lua
lua: lua-format

.PHONY: core
core:
	cd core && cargo build --release

# https://github.com/Koihik/LuaFormatter
.PHONY: lua-format
lua-format:
	find lua -name "*.lua"| xargs lua-format -i

.PHONY: vim
vim: vim-lint

.PHONY: vim-lint
vim-lint:
	vint --version
	vint plugin
	vint autoload

tools/ve/bin/vint: tools
	cd tools && python -m venv ve && ./ve/bin/pip install vim-vint

tools:
	mkdir -p $@
