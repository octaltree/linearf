" set verbose=1

let s:suite = themis#suite('test')
let s:assert = themis#helper('assert')
let s:dir = fnamemodify(resolve(expand('<sfile>:p')), ':h')
call luaeval("function(_A) package.path = table.concat({package.path, _A}, ';') end", s:dir .. "/test.lua")

function! s:suite.run() abort
  lua require('test').run()
endfunction
