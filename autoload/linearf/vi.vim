function! linearf#vi#_lua() abort
  if luaeval('jit and jit.version ~= nil')
    return 'luajit'
  else
    let v = split(luaeval('_VERSION'))[1]
    let label = join(split(v, '\.'), '')
    return 'lua' . label
  endif
endfunction
