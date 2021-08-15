function! Foo(n) abort
  "call nvim_buf_set_lines(0, 1, -1, v:true, s)
  "for i in range(1, a:n)
  "endfor
  let start = reltime()
  call map(range(1, a:n), 'string(v:val)')
  echo reltimestr(reltime(start))
endfunction

function! Load() abort
  let start = reltime()
  for i in range(1, 60)
    let line = line('.')
    execute 'e!'
    call cursor(line - 15, 1)
    call cursor(line, 1)
    echomsg line
    redraw!
  endfor
  "echo reltimestr(reltime(start))
endfunction

let s:i = 1
let s:line = 1
function! s:frame(_) abort
  let start = reltime()
  let line = line('.')
  execute 'e!'
  let s = join(map(range(1, line-1), {_ -> 'j'}), '')
  "call cursor(line-1, 1)
  call feedkeys(s)
  "redraw!
  "call cursor(line, 1)
  redraw!
  let s:i = s:i + 1
  "echomsg reltimestr(reltime(start))
endfunction
function! s:moved() abort
  let s:line = line('.')
endfunction

function! Start() abort
  "autocmd CursorMoved * call s:moved()
  call timer_start(100, funcref('s:frame'), {'repeat': 100})
endfunction
