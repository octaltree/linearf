local M = {
    -- config
    try_build_if_not_loaded = false,
    try_build_on_error = false,
    -- mutables
    inner = false, -- need non nil value
    _build = function()
    end
}

local path = require('linearf.path')
local utils = require('linearf.utils')
local Result = require('linearf.result')

local function try_init()
    local success, mod = pcall(require, path.bridge_name_body())
    if not success then return false end
    M.inner = mod
    return true
end

M = setmetatable(M, {
    __index = function(self, key)
        if not self.inner then try_init() end
        if not self.inner and M.try_build_if_not_loaded then
            M._build()
            try_init()
        end
        if not self.inner then
            return function(...)
                return Result.Err('bridge is not loaded')
            end
        end
        return function(...)
            local result = Result.pcall(self.inner[key], ...)
            local ret = result:map_err(function(e)
                if type(e) == 'userdata' then
                    if self.inner.is_related_recipe(e) and M.try_build_on_error then
                        M._build()
                        -- TODO: reload
                        package.loaded['linearf.bridge'] = nil
                        try_init()
                        -- TODO: retry
                    end
                    return self.inner.format_error(key, e)
                else
                    return e
                end
            end)
            return ret
        end
    end
})

local function format_recipe(recipe)
    recipe = recipe or {}
    local crates = {}
    local sources = {}
    local matchers = {}
    local converters = {}
    for _, x in ipairs(recipe.crates or {}) do
        table.insert(crates, utils.dict(x))
    end
    for _, x in ipairs(recipe.sources or {}) do
        table.insert(sources, utils.dict(x))
    end
    for _, x in ipairs(recipe.matchers or {}) do
        table.insert(matchers, utils.dict(x))
    end
    for _, x in ipairs(recipe.converters or {}) do
        table.insert(converters, utils.dict(x))
    end
    return vim.fn.json_encode(utils.dict({
        crates = utils.list(crates),
        sources = utils.list(sources),
        matchers = utils.list(matchers),
        converters = utils.list(converters)
    }))
end

function M.build(recipe)
    local features = 'mlua/' .. utils.lua_ver()
    local json = format_recipe(recipe)
    utils.command('let $LINEARF_RECIPE = ' .. vim.fn.string(json))
    utils.command('let $RUSTFLAGS = "-Awarnings"')
    local tmp = vim.fn.getcwd()
    local t = table.concat({
        'cd %s; ',
        'cargo run --bin=bundle -- %s;',
        'cd %s;'
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

function M.init(build)
    path.append_bridge()
    M._build = build
    return try_init()
end

return M
