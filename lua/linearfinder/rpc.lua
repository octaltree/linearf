local uv = require('luv')
local vi = require('linearfinder.vi')
local path = require('linearfinder.path')

local function json_encode(data)
    local status, result = pcall(vim.fn.json_encode, data)
    if status then
        return result
    else
        return nil, result
    end
end

local function json_decode(s)
    vi.validate {s = {s, 's'}}
    local status, result = pcall(vim.fn.json_decode, s)
    if status then
        return result
    else
        return nil, result
    end
end

local function start(cmd, args)
    vi.validate {cmd = {cmd, 's'}, args = {args, 't'}}
    local stdin = uv.new_pipe(false)
    local stdout = uv.new_pipe(false)
    local stderr = uv.new_pipe(false)
    local handle, pid
    do
        local params = {args = args, stdio = {stdin, stdout, stderr}}
        local function on_exit(_code, _signal)
            stdin:close()
            stdout:close()
            stderr:close()
            handle:close()
        end
        handle, pid = uv.spawn(cmd, params, on_exit)
    end
    return {handle = handle, pid = pid}
end

return {start = start}
