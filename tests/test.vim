" set verbose=1

let s:suite = themis#suite('test')
let s:assert = themis#helper('assert')
let g:linearf#command = 'L'

function! s:suite.run() abort
  call linearf#build()
  call linearf#init()
  call linearf#run('')
endfunction
