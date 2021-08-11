" build
" rpc lua
" prompt, list, preview UI
" action

function! s:_build_core_shell() abort
  " TODO: read source paths
  return 'cd ' . shellescape(linearf#path#core()) . ' && cargo build --release'
endfunction

function! linearf#build() abort
  execute '! ' . s:_build_core_shell()
endfunction

function! linearf#ensure_build() abort
  execute 'silent ! ' . s:_build_core_shell()
endfunction

function! linearf#start() abort
  lua require('linearf').start()
endfunction
