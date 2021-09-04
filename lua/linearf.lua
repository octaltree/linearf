local M = {}

M.value = require('linearf.value')
local bridge

function M.init()
    local path = require('linearf.path')
    path.append_bridge()
    local success, mod = pcall(require, 'bridge')
    if success then bridge = mod end
    return success
end

-- local function echo_error(e) vi.call('linearf#_echo_error', e) end

function M.run()
    local v = M.value.finish()
    local result = bridge.run(v[1], v[2])
end

function M.start(flow)
    local result = bridge.start(flow)
    if result then
        return result
    else
        -- echo_error(string.format("Flow \"%s\" is not exist.", flow))
        return nil
    end
end

function M.terminate(sid) bridge.terminate(sid) end

function M.count(sid) return bridge.count(sid) end

return M
