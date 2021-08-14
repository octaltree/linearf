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
  let sh = "find /home/octaltree/workspace"
  let out = system(sh)
  let xs = split(out, '\n')
  "let json = json_encode(out)
  let start = reltime()
  "let x =  json_decode(json)
  let bytes = []
  for x in xs
    call add(bytes, libcall(g:linearf#root_dir .. '/autoload/linearf/base64/target/release/libbase64.so', 'base64_encode', x))
  endfor
  call luaeval("require('linearf').send(_A)", bytes)
  "call pyeval("(lambda x: [])(_A)", xs)

  "lua linearf = require('linearf')
  "for i in xs
  "  call luaeval("linearf.append(_A)", i)
  "endfor
  "call luaeval('linearf.build()')
  echomsg reltimestr(reltime(start))
endfunction
