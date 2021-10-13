local C = {}
local M = {cache = C}

local utils = require('linearf.utils')

function M.sep()
    return utils.is_windows() and '\\' or '/'
end

function M.join(parts)
    return table.concat(parts, M.sep())
end

-- has no trailing slash
function M.root()
    if C._root == nil then C._root = utils.g('linearf#root_dir') end
    return C._root
end

-- PRIVATE

function M.background_command()
    local exe = utils.is_windows() and 'linearf.exe' or 'linearf'
    return M.join {M.root(), 'core', 'target', 'release', exe}
end

function M.bridge()
    return M.root() .. M.sep() .. 'model'
end

function M.bridge_dest()
    return M.join {M.bridge(), 'target', utils.lua_ver()}
end

function M.bridge_name_body()
    return 'linearf_bridge'
end

function M.bridge_name()
    if utils.is_windows() then
        return M.bridge_name_body() .. '.dll'
    else
        return 'lib' .. M.bridge_name_body() .. '.so'
    end
end

function M.bridge_release_bin()
    return M.join {M.bridge(), 'target', 'release', M.bridge_name()}
end

function M.cpath()
    local name
    if utils.is_windows() then
        name = '?.dll'
    else
        -- mac?
        name = 'lib?.so'
    end
    local lua = M.join {M.root(), 'model', 'target', utils.lua_ver(), name}
    return lua
end

return M
