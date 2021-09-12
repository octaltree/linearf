let g:linearf#root_dir = fnamemodify(resolve(expand('<sfile>:p')), ':h:h')
let g:linearf#command = get(g:, 'linearf#command', 'Linearf')
let g:linearf#recipe = get(g:, 'linearf#recipe', v:null)
let g:linearf#view = get(g:, 'linearf#view', {})

let s:initialized = v:false

function! linearf#build() abort
  return linearf#path#build()
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

function! linearf#run(args) abort
  let selected = s:_get_visual()
  lua linearf = require('linearf')
  call luaeval('linearf.new()')
  call luaeval('linearf.value:push(_A)', selected)
  call luaeval('linearf.value:push(_A)', a:args)
  let session = luaeval('linearf.call("run")')
  call linearf#ui#start(session)
  return session
endfunction

function! linearf#resume(session) abort
  call linearf#ui#start(a:session)
endfunction

function! s:_get_visual() abort
  let tmp = @@
  silent normal! gvy
  let selected = @@
  let @@ = tmp
  return selected
endfunction

function! linearf#_echo_error(e) abort
  let s = type(a:e) ==# v:t_string ? a:e : string(a:e)
  let msg = printf('[linearf] %s', s)
  echohl Error | echomsg msg | echohl None
endfunction

function! linearf#_lua() abort
  if luaeval('jit and jit.version ~= nil')
    return 'luajit'
  endif
  let v = split(luaeval('_VERSION'))[1]
  let label = join(split(v, '\.'), '')
  return 'lua' . label
endfunction

" Returns 0 if the session is not found
function! linearf#fetch_num(session) abort
endfunction

" Returns v:null if the session is not found.
" If the result is less than the range, it does not fail.
function! linearf#fetch(session, range) abort
  " "id,view\n"
endfunction
