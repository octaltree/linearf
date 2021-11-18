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
```vim
lua<<EOF
local linearf = require('linearf')
local flavors = require('linearf_my_flavors')

-- Initialize with a view module
linearf.init(require('linearf-vanilla').new())

linearf.recipe.sources = {
    {
        name = "identity",
        path = "flavors_plain::Identity"
    },
    {
        name = "command",
        path = "flavors_tokio::Command"
    }
}
linearf.recipe.matchers = {
    {
        name = "identity",
        path = "flavors_plain::Identity"
    },
    {
        name = "substring",
        path = "flavors_plain::Substring"
    }
}
linearf.recipe.converters = {
    {
        name = "format_line",
        path = "flavors_plain::FormatLine"
    }
}

-- Define senarios with presets and your preferences
linearf.senarios['line'] = flavors.merge(flavors.senarios['line'], {
    linearf = {
        list_nnoremap = {
            ["<CR>"] = flavors.hide_and(flavors.actions.line.jump)
        }
    },
    view = {
        querier_on_start = 'insert'
    }
})
linearf.senarios['file'] = flavors.merge(flavors.senarios['file_find'], {})
linearf.senarios['grep'] = flavors.merge(flavors.senarios['grep_rg'], {})
linearf.context_managers['line'] = flavors.context_managers['line']
linearf.context_managers['file'] = flavors.context_managers['file_find']
linearf.context_managers['grep'] = flavors.context_managers['grep_rg']

-- Auto building if you want
linearf.bridge.try_build_if_not_loaded = true
linearf.bridge.try_build_on_error = true
EOF

nnoremap <Denite>/ :<c-u>lua lnf('line')<CR>
au FileType linearf-vanilla-querier call s:bind_linearf_vanilla_querier()
function! s:bind_linearf_vanilla_querier() abort
  nmap <silent><buffer><nowait>q <Plug>(linearf-hide)
  nmap <silent><buffer><CR> <Plug>(linearf-goto-list)
  imap <silent><buffer><esc> <Plug>(linearf-goto-list)
endfunction
au FileType linearf-vanilla-list call s:bind_linearf_vanilla_list()
function! s:bind_linearf_vanilla_list() abort
  nmap <silent><buffer><nowait>q <Plug>(linearf-hide)
  nmap <silent><buffer>i <Plug>(linearf-goto-querier-insert)
  nmap <silent><buffer>I <Plug>(linearf-goto-querier-insert)
  nmap <silent><buffer>a <Plug>(linearf-goto-querier-insert)
  nmap <silent><buffer>A <Plug>(linearf-goto-querier-insert)
  nmap <silent><buffer>o <Plug>(linearf-goto-querier-insert)
  nmap <silent><buffer>O <Plug>(linearf-goto-querier-insert)
endfunction
```

Then run with the pre-defined senario and its difference.
```vim
lua lnf('line')
lua lnf('line', {})
lua lnf({})
```
For more information, see `:help linearf`

## TODO
- [x] implement logic
- [x] runtime reloading and auto building
- [x] implement view
- [x] implement action
- [ ] implement linearf-my-flavors
- [ ] use vim as a fuzzy finder from CLI
- [ ] implement preview
