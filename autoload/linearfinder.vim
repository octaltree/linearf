" build
" rpc lua
" prompt, list, preview UI
" action

function! linearfinder#build() abort
  " TODO: read source paths
  let sh = 'cd ' . shellescape(linearfinder#path#core()) . ' && cargo build --release'
  execute '! ' . sh
endfunction

function! linearfinder#start() abort
  lua <<EOF
  local main = require('linearfinder')
  main.start()
EOF
endfunction
