# linearf
A fast and extensible fuzzy finder for vimmers

![](https://user-images.githubusercontent.com/7942952/144457920-fea501bd-196a-4192-99d6-7c1658d9dd0b.png)

## Concept
* Show the first view faster
* Find as fast as if they were in linear time
* High modularity and extensibility

## Requirements
* [cargo](https://doc.rust-lang.org/book/ch01-01-installation.html) nightly
* vim and +lua/dyn, or neovim and luajit

## Usage
First, install the plugins and sources locally. If you use dein as your package
manager, it will look like this.
```vim
call dein#add('octaltree/linearf', {'merged': 0})
call dein#add('octaltree/linearf-my-flavors') # optional
```

Paste config to your config.lua
```lua
local linearf = require('linearf')
local flavors = require('linearf-my-flavors')

-- Initialize with a view module
linearf.init(require('linearf-vanilla').new())

-- Specify the sources to include in the build
linearf.recipe.sources = {
    {name = "identity", path = "flavors_plain::Identity"},
    {name = "command", path = "flavors_tokio::Command"}
}
linearf.recipe.converters = {
    {name = "format_line", path = "flavors_plain::FormatLine"}
}
linearf.recipe.matchers = {
    {name = "identity", path = "flavors_plain::Identity"},
    {name = "substring", path = "flavors_plain::Substring"},
    {name = "clap", path = "flavors_clap::Clap"}
}
-- Auto-build if you want
linearf.bridge.try_build_if_not_exist = true
linearf.bridge.try_build_on_error = true

-- Define your scenario. flavors provides you with several presets
local function set(target, context_manager, senario)
    linearf.context_managers[target] = context_manager
    linearf.senarios[target] = senario
end
set('line', flavors.context_managers['line'], flavors.merge {
    flavors.senarios['line'],
    flavors.senarios.quit,
    flavors.senarios.no_list_insert,
    flavors.senarios.no_querier_normal,
    {
        linearf = {
            list_nnoremap = {
                ["<CR>"] = flavors.hide_and(flavors.actions.line.jump)
            },
            querier_inoremap = {
                ["<CR>"] = flavors.normal_and(
                    flavors.hide_and(flavors.actions.line.jump))
            }
        },
        view = {querier_on_start = 'insert'}
    }
})
local use_rg = false
set('file', flavors.context_managers[use_rg and 'file_rg' or 'file_find'],
    flavors.merge {
    flavors.senarios[use_rg and 'file_rg' or 'file_find'],
    flavors.senarios.quit,
    flavors.senarios.no_list_insert,
    flavors.senarios.no_querier_normal,
    {
        linearf = {
            list_nnoremap = {
                ["<CR>"] = flavors.hide_and(flavors.actions.file.open),
                ["<nowait>s"] = flavors.hide_and(flavors.actions.file.split),
                ["t"] = flavors.hide_and(flavors.actions.file.tabopen),
                ["v"] = flavors.hide_and(flavors.actions.file.vsplit)
            },
            querier_inoremap = {
                ["<CR>"] = flavors.normal_and(
                    flavors.hide_and(flavors.actions.file.open))
            }
        }
    }
})
set('grep', flavors.context_managers[use_rg and 'grep_rg' or 'grep_grep'],
    flavors.merge {
    flavors.senarios[use_rg and 'grep_rg' or 'grep_grep'],
    flavors.senarios.quit,
    flavors.senarios.no_list_insert,
    flavors.senarios.enter_list,
    {
        linearf = {
            list_nnoremap = {
                ["<CR>"] = flavors.hide_and(flavors.actions.grep.open),
                ["<nowait>s"] = flavors.hide_and(flavors.actions.grep.split),
                ["t"] = flavors.hide_and(flavors.actions.grep.tabopen),
                ["v"] = flavors.hide_and(flavors.actions.grep.vsplit)
            },
            querier_inoremap = {},
            querier_nnoremap = {
                ["<nowait><ESC>"] = flavors.actions.view.goto_list
            }
        }
    }
})

-- optional
linearf.utils.command(
    "nnoremap <silent><space>/ :<c-u>lua linearf.run('line')<CR>")
linearf.utils.command(
    "nnoremap <silent><space>f :<c-u>lua linearf.run('file')<CR>")
linearf.utils.command(
    "nnoremap <silent><space>g :<c-u>lua linearf.run('grep')<CR>")
EOF
```

<details>
<summary>or vimrc</summary>

Just surround it with EOF.
```vim
" lua block in vim script
lua<<EOF
local linearf = require('linearf')
local flavors = require('linearf-my-flavors')

-- Initialize with a view module
linearf.init(require('linearf-vanilla').new())

-- Specify the sources to include in the build
linearf.recipe.sources = {
    {name = "identity", path = "flavors_plain::Identity"},
    {name = "command", path = "flavors_tokio::Command"}
}
linearf.recipe.converters = {
    {name = "format_line", path = "flavors_plain::FormatLine"}
}
linearf.recipe.matchers = {
    {name = "identity", path = "flavors_plain::Identity"},
    {name = "substring", path = "flavors_plain::Substring"},
    {name = "clap", path = "flavors_clap::Clap"}
}
-- Auto-build if you want
linearf.bridge.try_build_if_not_exist = true
linearf.bridge.try_build_on_error = true

-- Define your scenario. flavors provides you with several presets
local function set(target, context_manager, senario)
    linearf.context_managers[target] = context_manager
    linearf.senarios[target] = senario
end
set('line', flavors.context_managers['line'], flavors.merge {
    flavors.senarios['line'],
    flavors.senarios.quit,
    flavors.senarios.no_list_insert,
    flavors.senarios.no_querier_normal,
    {
        linearf = {
            list_nnoremap = {
                ["<CR>"] = flavors.hide_and(flavors.actions.line.jump)
            },
            querier_inoremap = {
                ["<CR>"] = flavors.normal_and(
                    flavors.hide_and(flavors.actions.line.jump))
            }
        },
        view = {querier_on_start = 'insert'}
    }
})
local use_rg = false
set('file', flavors.context_managers[use_rg and 'file_rg' or 'file_find'],
    flavors.merge {
    flavors.senarios[use_rg and 'file_rg' or 'file_find'],
    flavors.senarios.quit,
    flavors.senarios.no_list_insert,
    flavors.senarios.no_querier_normal,
    {
        linearf = {
            list_nnoremap = {
                ["<CR>"] = flavors.hide_and(flavors.actions.file.open),
                ["<nowait>s"] = flavors.hide_and(flavors.actions.file.split),
                ["t"] = flavors.hide_and(flavors.actions.file.tabopen),
                ["v"] = flavors.hide_and(flavors.actions.file.vsplit)
            },
            querier_inoremap = {
                ["<CR>"] = flavors.normal_and(
                    flavors.hide_and(flavors.actions.file.open))
            }
        }
    }
})
set('grep', flavors.context_managers[use_rg and 'grep_rg' or 'grep_grep'],
    flavors.merge {
    flavors.senarios[use_rg and 'grep_rg' or 'grep_grep'],
    flavors.senarios.quit,
    flavors.senarios.no_list_insert,
    flavors.senarios.enter_list,
    {
        linearf = {
            list_nnoremap = {
                ["<CR>"] = flavors.hide_and(flavors.actions.grep.open),
                ["<nowait>s"] = flavors.hide_and(flavors.actions.grep.split),
                ["t"] = flavors.hide_and(flavors.actions.grep.tabopen),
                ["v"] = flavors.hide_and(flavors.actions.grep.vsplit)
            },
            querier_inoremap = {},
            querier_nnoremap = {
                ["<nowait><ESC>"] = flavors.actions.view.goto_list
            }
        }
    }
})

-- optional
linearf.utils.command(
    "nnoremap <silent><space>/ :<c-u>lua linearf.run('line')<CR>")
linearf.utils.command(
    "nnoremap <silent><space>f :<c-u>lua linearf.run('file')<CR>")
linearf.utils.command(
    "nnoremap <silent><space>g :<c-u>lua linearf.run('grep')<CR>")
EOF
```
</details>

Then run with the pre-defined senario and its difference.
```vim
lua lnf('line')
lua lnf('line', {})
```
For more information, see `:help linearf`

## TODO
- [x] implement logic
- [x] runtime reloading and auto building
- [x] implement view
- [x] implement action
- [x] implement linearf-my-flavors
- [ ] use vim as a fuzzy finder from CLI
- [ ] implement preview
