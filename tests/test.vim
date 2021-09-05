" set verbose=1

let s:suite = themis#suite('test')
let s:assert = themis#helper('assert')
let g:linearf#command = 'Lnf'

function! s:suite.run() abort
  call linearf#build()
  call linearf#init()
  let session = linearf#run('')
  call s:assert.true(session > 0)
  Lnf a
endfunction
