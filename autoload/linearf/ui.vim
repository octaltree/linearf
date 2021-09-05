function! linearf#ui#init() abort
  execute
        \ 'command! -nargs=+ -range -bar'
        \ g:linearf#command
        \ 'call linearf#run(<q-args>)'
endfunction

function! linearf#ui#_get_visual() abort
  let tmp = @@
  silent normal! gvy
  let selected = @@
  let @@ = tmp
  return selected
endfunction

function! linearf#ui#start() abort
endfunction
