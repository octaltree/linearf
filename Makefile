.PHONY: core format-lua

all: core format-lua

core:
	cd core && cargo build --release

# https://github.com/Koihik/LuaFormatter
format-lua:
	find lua -name "*.lua"| xargs lua-format -i
