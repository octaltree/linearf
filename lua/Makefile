.PHONY: dev
dev: format lint

# https://github.com/Koihik/LuaFormatter
.PHONY: format
format:
	@find -name "*.lua"| xargs -i sh -c 'lua-format --check "{}" -c lua_format|| lua-format -i "{}" -c lua_format'

# https://github.com/mpeterv/luacheck
.PHONY: lint
lint:
	@find -name "*.lua"| xargs luacheck -q |\
		sed '/accessing undefined variable \[0m\[1mvim/d' |\
		sed '/unused argument \[0m\[1m_/d' |\
		sed '/^$$/d' |\
		sed '/^Checking/d' |\
		sed '/^Total:/d'
