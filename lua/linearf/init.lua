local M = {recipe = nil, wm = nil}

local bridge = require('linearf.bridge')
local utils = require('linearf.utils')

function M.build()
    return bridge.build(M.recipe)
end

function M.init(wm)
    bridge.init()
    M.wm = wm
end

function M.run(senario_name)
    local result = bridge.run(senario_name)
    local session
    if not result.ok then
        utils.echo_error(result.value)
        error(result.value)
    else
        session = result.value
    end
    M.wm:start(session)
end

function M.resume(session)
    M.wm:start(session)
end

return M
