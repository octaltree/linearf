# linearfinder.vim
Fast and extensible fuzzy finder

## Concepts
* fzf is not vim
* Respect denite.nvim
  - Value extensiblity
* Show the first view faster
* Respond as fast as if they were in linear time even for huge sources
* Use vim as a fuzzy finder

## Requirements
* cargo
* pip
* vim if_lua or neovim lua

## Installation
For dein
```vim
call dein#add('octaltree/vimrocks')
call dein#add('octaltree/linearfinder.vim')

lua <<EOF
local vimrocks = require('vimrocks')
if not vimrocks.luarocks_installed() then
  vimrocks.local_install_luarocks()
end
EOF
let g:vimrocks#enable_at_startup = 1
```

After installing vimrocks, You need build linearfinder.
```
call linearfinder#build()
```

## Configuration
```
```
For more information, see help
