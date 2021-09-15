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

function! linearf#run(senario_diff, action_diff) abort
  let selected = s:get_visual()
  lua linearf = require('linearf')
  call luaeval('linearf.new()')
  call luaeval('linearf.value:push(_A)', selected)
  call luaeval('linearf.value:push(_A)', a:args)
  let session = luaeval('linearf.call("run")')
  if session
    call linearf#ui#start(session, 1)
  endif
  return session
endfunction

function! linearf#resume(session) abort
  let flow = luaeval('linearf.call_one("resume", _A)', a:session)
  if flow
    call linearf#ui#start(a:session, flow)
  endif
  return flow
endfunction

function! linearf#feed_query() abort
  let s:flow = 42
  return s:flow
endfunction

function! linearf#in_progress() abort
endfunction

" TODO: filtered num, sourced num
" Returns 0 if the session is not found
function! linearf#fetch_num() abort
endfunction

" Returns v:null if the session is not found.
" If the number of items is less than [start, end), the result is less than end-start
function! linearf#fetch_view(start, end) abort
  " "id,view\n"
endfunction

" TODO: preview will overuse this.
" If an id is not found, fail and return v:null
function! linearf#fetch_value(ids) abort
endfunction

function! linearf#_run(args) abort
  return linearf#run(request)
endfunction

function! s:get_visual() abort
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
