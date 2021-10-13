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
            return function()
                return Result.Err('bridge is not loaded')
            end
        end
        return function(...)
            local result = Result.pcall(self.inner[key], ...)
            local ret = result:map_err(function(e)
                if type(e) == 'userdata' then
                    if M.try_build_on_error and self.inner.is_related_recipe(e) then
                        M._build()
                        -- TODO: reload
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
    local timestamp = os.time(os.date("!*t"))
    do -- set args
        local json = format_recipe(recipe)
        utils.command('let $LINEARF_RECIPE = ' .. vim.fn.string(json))
        --utils.command('let $LINEARF_BRIDGE_SUFFIX = ' .. vim.fn.string(timestamp))
        utils.command('let $RUSTFLAGS = "-Awarnings"')
    end
    do -- compile
        local features = 'mlua/' .. utils.lua_ver()
        local cwd = vim.fn.getcwd()
        local dir = vim.fn.shellescape(path.bridge())
        local sh = vim.fn.printf(table.concat({
            'cd %s; ',
            'cargo run --bin=bundle -- %s;',
            'cd %s;'
        }, ''), dir, features, cwd)
        utils.command('! ' .. sh)
    end
    do -- deploy
        local bin = path.bridge_release_bin()
        local dir = path.bridge_dest()
        local name = path.bridge_name()
        vim.fn.mkdir(dir, 'p')
        vim.fn.rename(bin, path.join {dir, name})
    end
end

function M.init(build)
    if not string.find(package.cpath, path.cpath(), 0, true) then
        package.cpath = table.concat({package.cpath, path.cpath()}, ';')
    end
    M._build = build
    return try_init()
end

return M
