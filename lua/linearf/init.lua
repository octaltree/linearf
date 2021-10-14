local M = {
    -- reexport
    utils = require('linearf.utils'),
    path = require('linearf.path'),
    bridge = require('linearf.bridge'),
    -- config
    recipe = {crates = {}, sources = {}, matchers = {}, converters = {}},
    senarios = {},
    context_managers = {},
    -- mutables
    view = nil,
    _sessions = {}
}

local Session = require('linearf.session')
local Flow = require('linearf.flow')
local SenarioBuilder = require('linearf.senario_builder')
-- init stateful
-- bridge stateful
-- path cache
-- utils cache
-- result pure type
-- session type
-- flow type
-- senario_builder type

function M.build()
    return M.bridge.build(M.recipe)
end

function M.init(view)
    if M.view then
        M.view:destruct()
        M.utils.cache = {}
        M.path.cache = {}
        M._sessions = {}
    end
    _G['linearf'] = M
    M.bridge.init(M.build)
    M.view = view
end

local function new_senario_builder(senario_name, diff)
    local base = M.senarios[senario_name]
    if not base then base = {} end
    local c = M.context_managers[senario_name]
    if type(c) ~= 'function' then
        c = function()
            return nil
        end
    end
    return SenarioBuilder.new(base, c, diff)
end

function M.run(senario_name, diff)
    local senario_builder = new_senario_builder(senario_name, diff)
    local senario = senario_builder:build()
    local id = M.bridge.run(senario):unwrap()
    local sid = id.session
    local fid = id.flow
    local flow = Flow.new(sid, fid, senario)
    local sess = Session.new(sid, senario_builder):insert(fid, flow)
    M._sessions[sid] = sess
    M.view:flow(flow)
end

local function expect_session(sid)
    local sess = M._sessions[sid]
    if not sess then error(string.format("session %d is not found", sid)) end
    return sess
end

function M.query(session_id, q)
    local sess = expect_session(session_id)
    local senario = sess.senario_builder:build()
    senario.linearf.query = q
    local id = M.bridge.tick(session_id, senario):unwrap()
    local sid = id.session
    local fid = id.flow
    local flow = Flow.new(sid, fid, senario)
    sess:insert(fid, flow)
    M.view:flow(flow)
end

local function expect_flow(sid, fid)
    local sess = expect_session(sid)
    local flow = sess.flows[fid]
    if not flow then error(string.format("flow %d is not found", fid)) end
    return flow
end

function M.resume(session_id)
    local fid = M.bridge.resume(session_id):unwrap()
    local flow = expect_flow(session_id, fid)
    M.view:flow(flow)
end

function M.remove_session(session_id)
    M.bridge.remove_session(session_id):unwrap()
    M._sessions[session_id] = nil
end

return setmetatable(M, {
    __call = function(self, ...)
        local args = {...}
        local len = #args
        if len == 1 then
            local t = type(args[1])
            if t == 'string' then
                return self.run(args[1], {})
            elseif t == 'table' then
                return self.run('', args[1])
            end
        end
        if len == 2 then
            local name = args[1]
            local diff = args[2]
            if type(name) == 'string' and type(diff) == 'table' then
                return self.run(name, diff)
            end
        end
        error(table.concat({
            '`linearf` accepts args one of following signatures',
            'linearf(name: string)',
            'linearf(diff: table)',
            'linearf(name: string, diff: table)'
        }, '\n'))
    end
})
