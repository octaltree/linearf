local M = {}
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

function M.call(key) return function() return bridge[key](unpack(M.value)) end end

return M
