function! s:sep() abort
  if has('win32')
    return '\'
  else
    return '/'
  endif
endfunction

function! linearfinder#path#core() abort
  return g:linearfinder#root_dir . s:sep() . 'core'
endfunction
