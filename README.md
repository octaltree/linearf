# linearf
A fast and extensible fuzzy finder for vimmers

## Concepts
* Show the first view faster
* Find as fast as if they were in linear time
* modulality and extensibility
* fzf is not vim
* Use vim as a fuzzy finder from CLI

## Requirements
* [cargo](https://doc.rust-lang.org/book/ch01-01-installation.html)
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
