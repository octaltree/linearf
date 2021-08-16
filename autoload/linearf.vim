" build
" rpc lua
" prompt, list, preview UI
" action
let g:linearf#root_dir = fnamemodify(resolve(expand('<sfile>:p')), ':h:h')
let s:started = v:false
let s:session = v:null

function! linearf#build() abort
  let dir = linearf#path#bridge()
  let sh = printf('cd %$s && cargo build --release', shellescape(dir))
  execute '! ' . sh
endfunction

function! linearf#init() abort
  if s:started
    return
  endif
  let s:started = v:true
  lua require('linearf').init()
endfunction

function! linearf#start(flow) abort
  let s:session = luaeval('require("linearf").start(_A)', a:flow)
endfunction

function! s:resume(session) abort
endfunction

function! s:confirm() abort
endfunction

function! s:select() abort
endfunction

function! s:fetch(range) abort
  " "idx,view\n"
endfunction

function! s:fetch_num() abort
endfunction
