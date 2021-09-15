" set verbose=1

let s:suite = themis#suite('test')
let s:assert = themis#helper('assert')

function! s:suite.build() abort
  lua linearf = require('linearf')
  lua linearf.build()
endfunction

function! s:suite.init() abort
  lua linearf = require('linearf')
  lua linearf.init(require('linearf-vanilla').new())
endfunction

function! s:suite.run() abort
  lua linearf = require('linearf')
  lua linearf.init(require('linearf-vanilla').new())
  lua linearf.run()
endfunction
