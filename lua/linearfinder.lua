local uv = vim.loop
local path = require('linearfinder.path')

local function start()
    -- local stdin = uv.new_pipe(false)
    -- local stdout = uv.new_pipe(false)
    -- local stderr = uv.new_pipe(false)
    local handle, pid
    local spawn_params = {args = {}, stdio = {stdin, stdout, stderr}}
    local function onexit(code, signal) end
    local cmd = path.background_command()
    handle, pid = uv.spawn(cmd, spawn_params, onexit)
end

return {start = start, build = build}
