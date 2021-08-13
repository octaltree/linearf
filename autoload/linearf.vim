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

function! linearf#tmp() abort
  let sh = "find /home/octaltree/workspace| sed '/linearf/d'"
  let out = system(sh)
  let xs = split(out, '\n')
  let start = reltime()
  let json = json_encode(out)
  "let x =  json_decode(json)
  call luaeval("require('linearf').send(_A)", json)
  "call pyeval("(lambda x: [])(_A)", xs)

  "lua linearf = require('linearf')
  "for i in xs
  "  call luaeval("linearf.append(_A)", i)
  "endfor
  "call luaeval('linearf.build()')
  echomsg reltimestr(reltime(start))
endfunction
