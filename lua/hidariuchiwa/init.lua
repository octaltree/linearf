local hidariuchiwa = {}

function hidariuchiwa.main()
  local a = {}
  for i=1,300000 do
    a[i] = '' .. i
  end
  local start = os.clock()
  local bufnr = vim.api.nvim_get_current_buf()
  write(bufnr, a)
  print(os.clock() - start)
end

function write(bufnr, a)
  vim.api.nvim_buf_set_lines(bufnr, 0, -1, true, a)
end

return hidariuchiwa
