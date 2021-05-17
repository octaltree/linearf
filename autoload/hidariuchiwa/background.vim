function! hidariuchiwa#background#spawn() abort
  let argv = [g:hidariuchiwa#root_dir .. "/background/target/debug/background"]
  let maybe_job_id = jobstart(argv)
  if maybe_job_id <= 0
    throw "Failed to spawn"
  endif
  " job-id is a valid channel-id
  let s:job = maybe_job_id
endfunction
