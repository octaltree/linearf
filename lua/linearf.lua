local M = {}

local path = require('linearf.path')

function M.start()
    path.append_bridge()
    local bridge = require('bridge')
    bridge.spawn()
end

return M
