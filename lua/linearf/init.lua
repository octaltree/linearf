local M = {
    bridge = require('linearf.bridge'),
    recipe = {
        crates = {}
    },
    view = nil,
    senarios = {},
    context_managers = {},
    sessions = {}
}

local utils = require('linearf.utils')
local Session = require('linearf.session')
local SenarioBuilder = require('linearf.senario_builder')

function M.build()
    return M.bridge.build(M.recipe)
end

function M.init(view)
    M.bridge.init()
    M.view = view
end

local function new_senario_builder(senario_name, diff)
    local base = M.senarios[senario_name]
    if not base then
        local s = string.format('senario "%s" is not found', senario_name)
        error(s)
    end
    local c = M.context_managers[senario_name]
    local cm
    if type(c) == 'function' then
        cm = c
    else
        cm = function()
            return nil
        end
    end
    return SenarioBuilder.new(base, cm, diff)
end

-- Senario:
--   * linearf: LinearfVars,
--   * source: SourceParams,
--   * matcher: MatcherParams,
--   * view: ViewParams,
-- LinearfVars:
--   * source: string
--   * matcher: string
--   * query: string
function M.run(senario_name, diff)
    local senario_builder = new_senario_builder(senario_name, diff)
    local senario = senario_builder:build()
    local id = M.bridge.run(senario):unwrap()
    local session = Session.new(M.bridge, id, senario, senario_builder)
    M.sessions[id] = session
    M.view:start(session)
end

function M.resume(session_id)
    M.view:start(session_id)
end

return M
