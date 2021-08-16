# linearf
A fast and extensible fuzzy finder for vimmers

## Concepts
* fzf is not vim
* Respect denite.nvim
  - Value extensiblity
* Show the first view faster
* Respond as fast as if they were in linear time even for huge sources
* Use vim as a fuzzy finder from CLI

## Requirements
* cargo
* +lua/dyn for vim / luajit for neovim

## Installation
For dein
```vim
call dein#add('octaltree/linearf')

call linearf#build()
```

## Configuration
```
call lienarf#init()
```
For more information, see help
