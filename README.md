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

Write config file
```lua
lua<<EOF
local linearf = require('linearf')
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
linearf.init(require('linearf-vanilla').new())
EOF
```

Then build your own fuzzy finder.
```vim
lua require('linearf').build()
```

After initialize, you can use the global variable `linearf` and execute it.
You can set up frequently used scenarios in advance.
```vim
lua linearf.run('', {})
lua linearf.run('simple')
lua linearf.run('simple', {})
```
For more information, see help

## TODO
- [x] implement logic
- [ ] implement view and controller
- [ ] implement linearf-my-flavors
- [ ] Use vim as a fuzzy finder from CLI
