local Dim = require('linearf.dim')
local M = {
    -- reexport
    utils = require('linearf.utils'),
    path = require('linearf.path'),
    bridge = require('linearf.bridge'),
    -- config
    recipe = Dim.from({
        crates = {},
        sources = {},
        matchers = {},
        converters = {}
    }),
    senarios = Dim.new(),
    context_managers = Dim.new(),
    _debug = true,
    -- mutables
    view = nil,
    _sessions = Dim.new()
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
-- dim type

function M.build()
    return M.bridge.build(M.recipe)
end

function M.init(view)
    if M.view then
        M.view:destruct()
        M.utils.cache = {}
        M.path.cache = {}
        M._sessions = Dim.new()
    end
    _G['linearf'] = M
    _G['lnf'] = M
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
    return SenarioBuilder.new(M.view.DEFAULT, base, c, diff)
end

local function with_serializable(senario, f)
    local action = senario.action
    senario.action = nil
    local ret = f(senario)
    senario.action = action
    return ret
end

function M.run(senario_name, diff)
    if M._debug then M.utils.command("let g:_linearf_time = reltime()") end
    local senario_builder = new_senario_builder(senario_name, diff)
    local senario = senario_builder:build()
    local id = with_serializable(senario, function(s)
        return M.bridge.run(s):unwrap()
    end)
    local sid = id.session
    local fid = id.flow
    local flow = Flow.new(M.bridge, sid, fid, senario)
    local sess = Session.new(sid, senario_builder):insert(fid, flow)
    M._sessions:set(sid, sess)
    M.view:flow({awake = 'session'}, flow)
end

local function expect_session(sid)
    local sess = M._sessions[sid]
    if not sess then error(string.format("session %d is not found", sid)) end
    return sess
end

function M._query(session_id, q)
    if M._debug then M.utils.command("let g:_linearf_time = reltime()") end
    local sess = expect_session(session_id)
    local senario = sess.senario_builder:build()
    senario.linearf.query = q
    local id = with_serializable(senario, function(s)
        return M.bridge.tick(session_id, s):unwrap()
    end)
    local sid = id.session
    local fid = id.flow
    local flow = Flow.new(M.bridge, sid, fid, senario)
    sess:insert(fid, flow)
    M.view:flow({awake = 'flow'}, flow)
end

local function expect_flow(sid, fid)
    local sess = expect_session(sid)
    local flow = sess.flow[fid]
    if not flow then error(string.format("flow %d is not found", fid)) end
    return flow
end

function M.resume(session_id)
    if M._debug then M.utils.command("let g:_linearf_time = reltime()") end
    local fid = M.bridge.resume(session_id):unwrap()
    local flow = expect_flow(session_id, fid)
    M.view:flow({awake = 'resume'}, flow)
end

function M.remove_session(session_id)
    M.bridge.remove_session(session_id):unwrap()
    M._sessions[session_id] = nil
end

function M.execute_action(senario, items)
    local a
    if type(senario.linearf.action) == 'function' then
        a = senario.linearf.action
    else
        local name = senario.linearf.action
        a = M.actions[name]
        if type(a) ~= 'function' then
            error(string.format('Action "%s" is not found', name))
        end
    end
    senario.view = nil
    senario.source = nil
    senario.matcher = nil
    return a(senario, items)
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
