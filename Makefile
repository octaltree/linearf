PATH := ./tools/ve/bin:$(PATH)

.PHONY: build
build:
	cd core && make release

.PHONY: dev
dev: core lua vim

.PHONY: lint
lint: core vim-lint lua-lint

.PHONY: d
d:
	watchexec 'make lint'

.PHONY: lua
lua: lua-format lua-lint

.PHONY: core
core:
	cd core && make dev

# https://github.com/Koihik/LuaFormatter
.PHONY: lua-format
lua-format:
	find lua -name "*.lua"| xargs lua-format -i

# https://github.com/mpeterv/luacheck
.PHONY: lua-lint
lua-lint:
	find lua -name "*.lua"| xargs luacheck | sed '/accessing undefined variable \[0m\[1mvim/d' | sed '/unused argument \[0m\[1m_/d'

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
