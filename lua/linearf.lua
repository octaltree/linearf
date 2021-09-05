local M = {}
local vi = require('linearf.vi')
local path = require('linearf.path')
local Value = require('linearf.value')
local unpack = table.unpack or unpack
M.value = Value.new()

local bridge

function M.init()
    path.append_bridge()
    local success, mod = pcall(require, 'bridge')
    if not success then return false end
    bridge = mod
    return true
end

function M.new() M.value = Value.new() end

local function echo_error(e) vi.call('linearf#_echo_error', e) end

local function call(name, ...)
    local ok, result = pcall(bridge[name], ...)
    if ok then
        return result
    else
        local e = bridge.error(name, result)
        local msg = string.format("[bridge.%s] %s", name, e)
        echo_error(msg)
        return nil
    end
end

function M.call(name) return call(name, unpack(M.value:finish())) end

function M.call_one(name, arg) return call(name, arg) end

return M
