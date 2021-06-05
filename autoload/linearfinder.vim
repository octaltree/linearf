" build
" rpc lua
" prompt, list, preview UI
" action

function! s:_install_rocks() abort
  lua <<EOF
  local vimrocks = require('vimrocks')
  vimrocks.append_path()
  local lib = vimrocks.path.lualib(vimrocks.lua_version())
  if not vimrocks.vi.filereadable(vimrocks.path.join{lib, 'pb.so'}) and
      not vimrocks.vi.filereadable(vimrocks.path.join{lib, 'pb.dll'}) then
    vimrocks.luarocks('install lua-protobuf')
  end
  if not vimrocks.vi.filereadable(vimrocks.path.join{lib, 'luv.so'}) and
      not vimrocks.vi.filereadable(vimrocks.path.join{lib, 'luv.dll'}) then
    vimrocks.luarocks('install luv')
  end
EOF
endfunction

function! linearfinder#build() abort
  call s:_install_rocks()
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
