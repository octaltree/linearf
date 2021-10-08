local C = {}
local M = {
    cache = C
}

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

function M.bridge_name()
    return utils.is_windows() and 'linearf_bridge.dll' or 'liblinearf_bridge.so'
end

function M.bridge_release_bin()
    return M.join {M.bridge(), 'target', 'release', M.bridge_name()}
end

function M.append_bridge()
    local name
    -- TODO: mac
    if utils.is_windows() then
        name = '?.dll'
    else
        name = 'lib?.so'
    end
    local lua = M.join {M.root(), 'model', 'target', utils.lua_ver(), name}
    local debug = M.join {M.root(), 'model', 'target', 'debug', name}
    package.cpath = table.concat({package.cpath, lua, debug}, ';')
end

return M
