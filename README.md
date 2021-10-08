# linearf [WIP]
A fast and extensible fuzzy finder for vimmers

## Concept
* Show the first view faster
* Find as fast as if they were in linear time
* High modularity and extensibility
* Use vim as a fuzzy finder from CLI

## Requirements
* [cargo](https://doc.rust-lang.org/book/ch01-01-installation.html) nightly
* +lua/dyn for vim / luajit for neovim

## Installation
First, install the plugins and sources locally. If you use dein as your package
manager, it will look like this.
```
call dein#add('octaltree/linearf')
call dein#add('octaltree/linearf-my-flavors')
```
Then build your own fuzzy finder.
```
lua require('linearf').recipe.sources = {}
lua require('linearf').recipe.matchers = {}
lua require('linearf').recipe.converters = {}
lua require('linearf').build()
```
One build is required per recipe change.

## Usage
After the build is complete, initialize it with the UI module.
```
lua require('linearf').init(require('linearf-vanilla').new())
```
After initialize, you can use the global variable `linearf` and execute it.
```
lua linearf.run('', {})
```
You can set up frequently used scenarios in advance.
```
lua require('linearf').senarios = {
    \     simple = {
    \         linearf = {
    \             source = 'simple',
    \             matcher = 'substring'
    \         }
    \     },
    \     osstr = {
    \         linearf = {
    \             source = 'osstr',
    \             matcher = 'substring'
    \         }
    \     }
    \ }
lua linearf.run('simple')
lua linearf.run('simple', {})
```
For more information, see help

## TODO
- [x] implement logic
- [ ] implement view and controller
- [ ] implement linearf-my-flavors
- [ ] Use vim as a fuzzy finder from CLI
