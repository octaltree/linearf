function! s:sep() abort
  if has('win32')
    return '\'
  else
    return '/'
  endif
endfunction

function! linearf#path#core() abort
  return g:linearf#root_dir . s:sep() . 'core'
endfunction
