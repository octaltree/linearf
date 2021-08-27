local M = {}

local path = require('linearf.path')
local vi = require('linearf.vi')
local bridge

function M.init()
    path.append_bridge()
    bridge = require('bridge')
end

function M.start(flow)
    local result = bridge.start(flow)
    if result then
        return result
    else
        echo_error(string.format("Flow \"%s\" is not exist.", flow))
        return nil
    end
end

function M.terminate(sid) bridge.terminate(sid) end

function M.count(sid) return bridge.count(sid) end

function echo_error(e) vi.call('linearf#_echo_error', e) end

return M
