local M = {
    -- config
    recipe = {
        crates = {},
        sources = {},
        matchers = {},
        converters = {}
    },
    senarios = {},
    -- mutables
    bridge = require('linearf.bridge'),
    view = nil,
    context_managers = {},
    sessions = {}
}
M.senarios[''] = {}

local utils = require('linearf.utils')
local Session = require('linearf.session')
local SenarioBuilder = require('linearf.senario_builder')

function M.build()
    return M.bridge.build(M.recipe)
end

function M.init(view)
    _G['linearf'] = M
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

function M.run(senario_name, diff)
    local senario_builder = new_senario_builder(senario_name, diff)
    local senario = senario_builder:build()
    local id = M.bridge.run(senario):unwrap()
    local sid = id.session
    local fid = id.flow
    local session = Session.new(M.bridge, sid, senario, senario_builder)
    M.sessions[sid] = session
    M.view:start(session)
end

function M.resume(session_id)
    M.view:start(session_id)
end

return M
