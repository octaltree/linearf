local uv = vim.loop
local path = require('linearfinder.path')
local rpc = require('linearfinder.rpc')

local function start()
    local cmd = path.background_command()
    local args = {}
    return rpc.start(cmd, args)
end

return {start = start}
