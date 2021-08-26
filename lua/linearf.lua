local M = {}

local path = require('linearf.path')
local bridge

function M.init()
    path.append_bridge()
    bridge = require('bridge')
end

function M.start(flow)
    local result = bridge.start_session(flow)
    if result then
        return result
    else
        echo_error(string.format("Flow \"%s\" is not exist.", flow))
        return nil
    end
end

function echo_error(e) end

return M
