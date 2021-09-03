function! linearf#ui#init() abort
  execute
        \ 'command! -nargs=+ -range -bar'
        \ g:linearf#command
        \ 'call linearf#run(<q-args>)'
endfunction
