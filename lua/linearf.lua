local M = {}

local path = require('linearf.path')

function M.start()
    package.cpath = package.cpath .. ';' .. path.bridge_signature()
    local bridge = require('bridge')
    ----print(bridge.sum(2, 3))
    ----print('start')
    bridge.spawn()
end

return M
