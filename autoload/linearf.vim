" build
" rpc lua
" prompt, list, preview UI
" action

function! linearf#build() abort
  execute '! ' . s:build_core_shell()
endfunction

let s:started = v:false

function! linearf#start() abort
  if s:started
    return
  endif
  let s:started = v:true
  lua require('linearf').start()
endfunction

function! s:build_core_shell() abort
  " TODO: read source paths
  return 'cd ' . shellescape(linearf#path#core()) . ' && cargo build --release'
endfunction
