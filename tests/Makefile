.PHONY: dev
dev: test

export THEMIS_HOME := ../tools/vim-themis

.PHONY: test
test: ../tools/vim-themis ../tests/test.vim
	THEMIS_VIM=nvim THEMIS_ARGS="-e -s --headless -u vimrc" ../tools/vim-themis/bin/themis ../tests/test.vim
	THEMIS_VIM=vim THEMIS_ARGS="-e -s -u vimrc" ../tools/vim-themis/bin/themis ../tests/test.vim

.PHONY: clean
clean:
	rm -rf ../tools

../tools/py/bin/vint: ../tools/py/bin
	@cd ../tools && ./py/bin/pip install vim-vint

../tools/py/bin: ../tools
	@cd ../tools && python -m venv py

../tools/vim-themis: ../tools
	git clone https://github.com/thinca/vim-themis $@

../tools:
	mkdir -p $@
