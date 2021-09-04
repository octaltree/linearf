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

function M.call(key) return bridge[key](unpack(M.value:finish())) end

return M
