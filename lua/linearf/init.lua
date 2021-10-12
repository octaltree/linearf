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
    sessions = {}
}

local Session = require('linearf.session')
local SenarioBuilder = require('linearf.senario_builder')
local Result = require('linearf.result')
-- init stateful
-- bridge stateful
-- path cache
-- utils cache
-- result pure type
-- senario_builder pure type
-- session pure type

function M.build()
    return M.bridge.build(M.recipe)
end

function M.init(view)
    if M.view then
        M.view:close()
        M.utils.cache = {}
        M.path.cache = {}
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
    local session = Session.new(M.bridge, sid, senario, senario_builder)
    M.sessions[sid] = session
    M.view:start(session)
end

function M.resume(session_id)
    M.view:start(session_id)
end

local function signature_error()
    local msg = table.concat({
        '`linearf` accepts args one of following signatures',
        'linearf(name: string)',
        'linearf(diff: table)',
        'linearf(name: string, diff: table)'
    }, '\n')
    return Result.Err(msg)
end

function M.unload()
  print(vim.inspect(M.bridge.unload()))
  package.loaded['linearf_bridge'] = nil
  linearf.bridge.inner = false
  collectgarbage('collect')
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
        signature_error():unwrap()
    end
})
