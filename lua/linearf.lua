local M = {}

local path = require('linearf.path')
local bridge

function M.start()
    path.append_bridge()
    bridge = require('bridge')
    bridge.spawn()
end

function M.linearf(source)
    print(source)
    bridge.linearf(source)
end

return M
