local vi = require('linearfinder.vi')

local function sep() return vi.cached_is_windows() and '\\' or '/' end

local function join(parts) return table.concat(parts, sep()) end

-- has no trailing slash
local function root() return vi.g('linearfinder#root_dir') end

local function background_command()
    return join {root(), 'core', 'target', 'release', 'linearfinder'}
end

return {join = join, root = root, background_command = background_command}
