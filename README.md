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

## Usage
First, install the plugins and sources locally. If you use dein as your package
manager, it will look like this.
```vim
call dein#add('octaltree/linearf')
call dein#add('octaltree/linearf-my-flavors')
```

Paste config file
```lua
lua<<EOF
local linearf = require('linearf')
linearf.init(require('linearf-vanilla').new())

linearf.recipe.sources = {}
linearf.recipe.matchers = {}
linearf.recipe.converters = {}

linearf.senarios = {
    simple = {
        linearf = {
            source = 'simple',
            matcher = 'substring'
        }
    },
    osstr = {
        linearf = {
            source = 'osstr',
            matcher = 'substring'
        }
    }
}

linearf.bridge.try_build_if_not_loaded = true
linearf.bridge.try_build_on_error = true
EOF
```
Then run with the pre-defined senario and its difference.
```vim
lua linearf({})
lua linearf('simple')
lua linearf('simple', {})
```
For more information, see `:help linearf`

## TODO
- [x] implement logic
- [ ] implement view and controller
- [ ] implement linearf-my-flavors
- [ ] Use vim as a fuzzy finder from CLI
