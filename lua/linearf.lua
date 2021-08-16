local M = {}

local path = require('linearf.path')
local bridge

function M.init()
    path.append_bridge()
    bridge = require('bridge')
    bridge.spawn()
end

function M.start(flow) return bridge.start_session(flow) end

return M
