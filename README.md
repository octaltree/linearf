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
linearf.init(require('linearf-vanilla').new())

linearf.recipe.sources = {}
linearf.recipe.matchers = {}
linearf.recipe.converters = {}

linearf.senarios = {
    simple = {
        linearf = {
            source = 'simple',
            matcher = 'substring',
            querier_inoremap = {
                ["<CR>"] = function(items)
                    print(vim.inspect(items))
                    linearf.view:hide_all()
                end
            },
            list_nnoremap = {
                ["<CR>"] = function(items)
                    print(vim.inspect(items))
                    linearf.view:hide_all()
                end
            }
        }
    },
    osstr = {
        linearf = {
            source = 'osstr',
            matcher = 'substring',
            list_nnoremap = {
                ["<CR>"] = function(items)
                  linearf.utils.command(vim.fn.printf("e %s", items[1].value))
                  linearf.view:hide_all()
                end
            }
        }
    }
}

linearf.bridge.try_build_if_not_exist = true
linearf.bridge.try_build_on_error = true
EOF
au FileType linearf-vanilla-querier call s:bind_linearf_vanilla_querier()
function! s:bind_linearf_vanilla_querier() abort
  nmap <silent><buffer><nowait>q <Plug>(linearf-hide-all)
  nmap <silent><buffer><CR> <Plug>(linearf-goto-list)
  imap <silent><buffer><esc> <Plug>(linearf-goto-list)
endfunction
au FileType linearf-vanilla-list call s:bind_linearf_vanilla_list()
function! s:bind_linearf_vanilla_list() abort
  nmap <silent><buffer><nowait>q <Plug>(linearf-hide-all)
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
lua linearf({})
lua linearf('simple')
lua linearf('simple', {})
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
