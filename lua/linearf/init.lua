local M = {recipe = nil, view = nil, collectors = {}}

local bridge = require('linearf.bridge')
local utils = require('linearf.utils')

function M.build()
    return bridge.build(M.recipe)
end

function M.init(view)
    bridge.init()
    M.view = view
end

-- Senario:
--   * source: string
--   * match: string
--   * source_params: table
--   * match_params: table

-- Option: nil|
--   * query: string
function M.run(senario_name, option)
    if type(option) ~= 'table' then option = {} end
    local result = bridge.run(senario_name)
    local session
    if not result.ok then
        utils.echo_error(result.value)
        error(result.value)
    else
        session = result.value
    end
    M.view:start(session)
end

function M.resume(session_id)
    M.view:start(session_id)
end

M.senario = {
    set = function(name, t)
        -- bridge.set_senario(name, t)
    end
}

function M.get_source_name(session_id)
end

function M.emit(session_id, source_name, query)
    local f = M.collectors[source_name]
    local ctx = f and f() or nil
end

return M
