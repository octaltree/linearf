function! linearf#ui#init() abort
  execute
        \ 'command! -nargs=+ -range -bar'
        \ g:linearf#command
        \ 'call linearf#run(<q-args>)'
  let f = get(g:linearf#view, 'init')
  if f
    call f()
  endif
endfunction

function! linearf#ui#start(session) abort
  let f = get(g:linearf#view, 'start')
  if f
    call f()
  endif
endfunction
