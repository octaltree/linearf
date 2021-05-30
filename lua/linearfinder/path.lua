local function sep()
    return vim.loop.os_uname().sysname == "Windows" and "\\" or "/"
end

local function join(parts) return table.concat(parts, sep()) end

-- has no trailing slash
local function root() return vim.g['linearfinder#root_dir'] end

local function background_command()
    return join {root(), 'background', 'target', 'release', 'linearfinder'}
end

return {join = join, root = root, background_command = background_command}
