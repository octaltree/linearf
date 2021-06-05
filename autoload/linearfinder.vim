" build
" rpc lua
" prompt, list, preview UI
" action

function! s:_ensure_install_rocks() abort
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

function! s:_install_rocks() abort
  lua <<EOF
  local vimrocks = require('vimrocks')
  vimrocks.append_path()
  local lib = vimrocks.path.lualib(vimrocks.lua_version())
  vimrocks.luarocks('install lua-protobuf')
  vimrocks.luarocks('install luv')
EOF
endfunction

function! s:_build_core_shell() abort
  " TODO: read source paths
  return 'cd ' . shellescape(linearfinder#path#core()) . ' && cargo build --release'
endfunction

function! linearfinder#build() abort
  call s:_install_rocks()
  execute '! ' . s:_build_core_shell()
endfunction

function! linearfinder#ensure_build() abort
  call s:_ensure_install_rocks()
  execute 'silent ! ' . s:_build_core_shell()
endfunction

function! linearfinder#start() abort
  lua <<EOF
  local main = require('linearfinder')
  main.start()
EOF
endfunction
