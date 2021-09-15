local M = {
    inner = false -- need non nil value
}

local path = require('linearf.path')
local utils = require('linearf.utils')
local Result = require('linearf.result')

M = setmetatable(M, {
    __index = function(self, key)
        if not self.inner then
            return Result.Err('bridge is not initialized')
        end
        return function(...)
            local result = Result.pcall(self.inner[key], ...)
            local ret = result:map_err(function(e)
                local msg = self.inner.format_error(e)
                utils.echo_error(msg)
            end)
            return ret
        end
    end
})

local function format_recipe(recipe)
    recipe = recipe or {}
    local crates = {}
    local sources = {}
    local matches = {}
    for _, x in ipairs(recipe.crates or {}) do
        table.insert(crates, utils.dict(x))
    end
    for _, x in ipairs(recipe.sources or {}) do
        table.insert(sources, utils.dict(x))
    end
    for _, x in ipairs(recipe.matches or {}) do
        table.insert(matches, utils.dict(x))
    end
    return vim.fn.json_encode(utils.dict({
        crates = utils.list(crates),
        sources = utils.list(sources),
        matches = utils.list(matches)
    }))
end

function M.build(recipe)
    local features = 'mlua/' .. utils.lua_ver()
    if type(recipe) == 'table' then
        local json = format_recipe(recipe)
        utils.command('let $LINEARF_RECIPE = ' .. vim.fn.string(json))
    end
    utils.command('let $RUSTFLAGS = "-Awarnings"')
    local tmp = vim.fn.getcwd()
    local t = table.concat({
        'cd %s; ', 'git checkout registrar &&',
        'cargo run --bin=preprocessor &&',
        'cargo build --features=%s --release && ', 'git checkout registrar; ',
        'cd %s'
    }, '')
    local b = vim.fn.shellescape(path.bridge())
    local sh = vim.fn.printf(t, b, features, tmp)
    utils.command('! ' .. sh)
    do
        local dest = path.bridge_dest()
        local name = path.bridge_name()
        local bin = path.bridge_release_bin()
        vim.fn.mkdir(dest, 'p')
        vim.fn.rename(bin, path.join {dest, name})
    end
end

function M.init()
    path.append_bridge()
    local success, mod = pcall(require, 'bridge')
    if not success then return false end
    M.inner = mod
    return true
end

return M
