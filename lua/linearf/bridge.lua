local M = {
    -- config
    try_build_if_not_exist = false,
    try_build_on_error = false,
    -- mutables
    inner = false, -- need non nil value
    retry_count = 0,
    _build = function()
    end
}

local path = require('linearf.path')
local utils = require('linearf.utils')
local Result = require('linearf.result')

local function try_reload()
    local latest
    do
        local dir = path.bridge_dest()
        vim.fn.mkdir(dir, 'p')
        local files = utils.readdir(dir)
        local timestamps = {}
        for _, name in ipairs(files) do
            local m = name:match('(%d+)%.[a-z]+$')
            if m then table.insert(timestamps, tonumber(m)) end
        end
        table.sort(timestamps)
        if #files == 0 then
            return false
        else
            latest = timestamps[#timestamps]
        end
    end
    local name = path.bridge_name_body(latest)
    if package.loaded[name] then
        Result.pcall(M.inner.remove_all_sessions)
        return true
    end
    local success, mod = pcall(require, name)
    if not success then return false end
    if M.inner then Result.pcall(M.inner.remove_all_sessions_later) end
    M.inner = mod
    return true
end

M = setmetatable(M, {
    __index = function(self, key)
        if not self.inner then try_reload() end
        if not self.inner and self.try_build_if_not_exist then
            self._build()
            try_reload()
        end
        return function(...)
            if not self.inner then
                return Result.Err('bridge is not loaded')
            end
            local result = Result.pcall(self.inner[key], ...)
            local args = {...}
            local ret = result:or_else(function(err)
                if type(err) ~= 'userdata' then
                    return Result.Err(err)
                end
                local e = self.inner.inspect_error(key, err)
                if self.try_build_on_error and e.is_related_recipe then
                    self._build()
                    try_reload()
                    if self.retry_count < 1 then
                        self.retry_count = self.retry_count + 1
                        return self[key](utils.unpack(args))
                    else
                        self.retry_count = 0
                    end
                end
                return Result.Err(e.message)
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
        utils.command('let $LINEARF_BRIDGE_SUFFIX = ' ..
                          vim.fn.string(timestamp))
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
        local name = path.bridge_name(timestamp)
        vim.fn.delete(dir, "rf")
        vim.fn.mkdir(dir, 'p')
        vim.fn.rename(bin, path.join {dir, name})
    end
end

function M.init(build)
    if not string.find(package.cpath, path.cpath(), 0, true) then
        package.cpath = table.concat({package.cpath, path.cpath()}, ';')
    end
    M._build = build
    return try_reload()
end

return M
