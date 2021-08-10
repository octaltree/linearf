" build
" rpc lua
" prompt, list, preview UI
" action

function! s:_build_core_shell() abort
  " TODO: read source paths
  return 'cd ' . shellescape(linearfinder#path#core()) . ' && cargo build --release'
endfunction

function! linearfinder#build() abort
  execute '! ' . s:_build_core_shell()
endfunction

function! linearfinder#ensure_build() abort
  execute 'silent ! ' . s:_build_core_shell()
endfunction

function! linearfinder#start() abort
  lua require('linearfinder').start()
endfunction
