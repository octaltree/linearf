" build
" rpc lua
" prompt, list, preview UI
" action
let g:linearf#root_dir = fnamemodify(resolve(expand('<sfile>:p')), ':h:h')
let s:started = v:false

function! linearf#build() abort
  let dir = linearf#path#bridge()
  let sh = printf('cd %$s && cargo build --release', shellescape(dir))
  execute '! ' . sh
endfunction

function! linearf#start() abort
  if s:started
    return
  endif
  let s:started = v:true
  lua require('linearf').start()
endfunction

function! linearf#linearf(source) abort
  call luaeval('require("linearf").linearf(_A)', a:source)
endfunction
