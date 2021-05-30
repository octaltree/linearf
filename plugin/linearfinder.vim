let _ = exists('g:loaded_linearfinder') && finish
let g:loaded_linearfinder = 1

" has no trailing slash
let g:linearfinder#root_dir = fnamemodify(resolve(expand('<sfile>:p')), ':h:h')
