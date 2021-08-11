let _ = exists('g:loaded_linearf') && finish
let g:loaded_linearf = 1

" has no trailing slash
let g:linearf#root_dir = fnamemodify(resolve(expand('<sfile>:p')), ':h:h')
