local M = {}

local path = require('linearf.path')

function M.start()
    package.cpath = package.cpath .. ';' .. path.bridge_signature()
    local bridge = require('bridge')
    print(bridge.spawn())
end

return M
