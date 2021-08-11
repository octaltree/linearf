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

function M.bridge_signature()
    local name
    -- TODO: mac
    if vi.cache.is_windows() then
        name = '?.dll'
    else
        name = 'lib?.so'
    end
    return M.join {C.root(), 'bridge', 'target', 'release', name}
end

return M
