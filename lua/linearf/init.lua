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
    scenarios = Dim.new(),
    context_managers = Dim.new(),
    _debug = false,
    -- mutables
    view = nil,
    _sessions = Dim.new()
}
-- init stateful
-- bridge stateful
-- path cache
-- utils cache
-- result pure type
-- session type
-- flow type
-- scenario_builder type
-- dim type

function M.build()
    return M.bridge.build(M.recipe)
end

function M.init(view)
    M.utils.augroup('linearf_leave', {'au VimLeave * let g:_linearf_leave = 1'})
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

local function new_scenario_builder(scenario_name, diff, winid)
    local ScenarioBuilder = require('linearf.scenario_builder')
    local base = M.scenarios[scenario_name]
    if not base then base = {} end
    local c = M.context_managers[scenario_name]
    if type(c) ~= 'function' then
        c = function()
            return nil
        end
    end
    return ScenarioBuilder.new(M.view.DEFAULT, base, c, diff, winid)
end

function M.run(scenario_name, diff)
    if M._debug then M.utils.command("let g:_linearf_time = reltime()") end
    local target = M.view:orig_winid() or vim.fn.win_getid()
    local scenario_builder = new_scenario_builder(scenario_name, diff, target)
    local scenario = scenario_builder:for_session()
    local id = M.bridge.run(scenario):unwrap()
    local Session = require('linearf.session')
    local Flow = require('linearf.flow')
    local sid = id.session
    local fid = id.flow
    local flow = Flow.new(M.bridge, sid, fid, scenario)
    local sess = Session.new(sid, scenario_builder):insert(fid, flow)
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
    local scenario = sess.scenario_builder:for_flow()
    scenario.linearf.query = q
    local id = M.bridge.tick(session_id, scenario):unwrap()
    local Flow = require('linearf.flow')
    local sid = id.session
    local fid = id.flow
    local flow = Flow.new(M.bridge, sid, fid, scenario)
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

function M.resume_last()
    local sids = M.bridge.session_ids(true):unwrap()
    if #sids == 0 then error('Failed to resume because sessions is empty') end
    local sid = sids[1]
    return M.resume(sid)
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
