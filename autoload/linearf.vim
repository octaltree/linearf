" build
" rpc lua
" prompt, list, preview UI
" action
let g:linearf#root_dir = fnamemodify(resolve(expand('<sfile>:p')), ':h:h')
let g:linearf#command = 'Linearf'
let s:initialized = v:false
let s:session = v:null

function! linearf#build() abort
  let dir = linearf#path#bridge()
  let sh = printf('cd %s && cargo build --release', shellescape(dir))
  execute '! ' . sh
endfunction

function! linearf#init() abort
  if s:initialized
    return
  endif
  let success = luaeval("require('linearf').init()")
  if !success
    return
  endif
  call linearf#ui#init()
  let s:initialized = v:true
endfunction

function! linearf#start(flow) abort
  let s:session = luaeval('require("linearf").start(_A)', a:flow)
endfunction

function! linearf#_echo_error(e) abort
  let s = type(a:e) ==# v:t_string ? a:e : string(a:e)
  let msg = printf('[linearf] %s', s)
  echohl Error | echomsg msg | echohl None
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
