function! s:sep() abort
  if has('win32')
    return '\'
  else
    return '/'
  endif
endfunction

function! linearf#path#bridge() abort
  return g:linearf#root_dir . s:sep() . 'model'
endfunction

function! linearf#path#bridge_dest() abort
  return join([linearf#path#bridge(), 'target', linearf#vi#_lua()], s:sep())
endfunction

function! linearf#path#bridge_name() abort
  if has('win32')
    return 'bridge.dll'
  else
    return 'libbridge.so'
  endif
endfunction

function! linearf#path#build() abort
  let dir = linearf#path#bridge()
  let features = 'mlua/' . linearf#vi#_lua()
  if type(g:linearf#recipe) == v:t_dict
    let $LINEARF_RECIPE = json_encode(g:linearf#recipe)
  endif
  let t = 'cd %s;' .
        \ 'git checkout registrar/registrar &&' .
        \ 'rustup run nightly cargo run --bin=registrar-preprocessor &&' .
        \ 'rustup run nightly cargo build --features=%s --release &&' .
        \ 'git checkout registrar/registrar'
  let sh = printf(t, shellescape(dir), features)
  execute '! ' . sh
  call s:replace(dir)
endfunction

function! s:replace(bridge) abort
  let dest = linearf#path#bridge_dest()
  let name = linearf#path#bridge_name()
  let target = join([a:bridge, 'target', 'release', name], s:sep())
  call mkdir(dest, 'p')
  call rename(target, dest . s:sep() . name)
endfunction
