" set verbose=1

let s:suite = themis#suite('test')
let s:assert = themis#helper('assert')
let s:dir = fnamemodify(resolve(expand('<sfile>:p')), ':h')
call luaeval("(function(x) package.path = table.concat({package.path, require('linearf.path').join{x, '?.lua'}}, ';') end)(_A)", s:dir)

function! s:suite.run() abort
  lua require('test').run()
endfunction
