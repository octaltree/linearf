local C = {}
local M = {cache = C}

local vi = require('linearf.vi')

function M.sep() return vi.cache.is_windows() and '\\' or '/' end

function M.join(parts) return table.concat(parts, M.sep()) end

-- has no trailing slash
function C.root()
    if C._root == nil then C._root = vi.g('linearf#root_dir') end
    return C._root
end

function M.background_command()
    local exe = vi.cache.is_windows() and 'linearf.exe' or 'linearf'
    return M.join {M.root(), 'core', 'target', 'release', exe}
end

function M.append_bridge()
    local name
    -- TODO: mac
    if vi.cache.is_windows() then
        name = '?.dll'
    else
        name = 'lib?.so'
    end
    local lua = M.join {C.root(), 'bridge', 'target', vi._lua(), name}
    local debug = M.join {C.root(), 'bridge', 'target', 'debug', name}
    package.cpath = table.concat({package.cpath, lua, debug}, ';')
end

return M
