local M = {}

local vi = require('linearf.vi')

function M.sep() return vi.cache.is_windows() and '\\' or '/' end

function M.join(parts) return table.concat(parts, M.sep()) end

-- has no trailing slash
function M.root() return vi.g('linearf#root_dir') end

function M.background_command()
    return M.join {M.root(), 'core', 'target', 'release', 'linearf'}
end

return M
