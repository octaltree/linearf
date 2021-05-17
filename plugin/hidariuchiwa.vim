let _ = exists('g:loaded_hidariuchiwa') && finish
let g:loaded_hidariuchiwa = 1

let g:hidariuchiwa#root_dir = fnamemodify(resolve(expand('<sfile>:p')), ':h:h')
