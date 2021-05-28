function! linearfinder#background#spawn() abort
  let argv = [g:linearfinder#root_dir .. "/core/target/debug/linearfinder"]
  let maybe_job_id = jobstart(argv)
  if maybe_job_id <= 0
    throw "Failed to spawn"
  endif
  " job-id is a valid channel-id
  let s:job = maybe_job_id
endfunction
