local function has(name) return vim.fn.has(name) == 1 end

local is_nvim_cache
local function cached_is_nvim()
    if is_nvim_cache == nil then is_nvim_cache = has('nvim') end
    return is_nvim_cache
end

local is_windows_cache
local function cached_is_windows()
    if is_windows_cache == nil then is_windows_cache = has('win32') end
    return is_windows_cache
end

local function g(name)
    if cached_is_nvim() then
        return vim.g[name]
    else
        return vim.eval('g:' .. name)
    end
end

local function validate(table)
    if cached_is_nvim() then return vim.validate(table) end
    -- Use only as a type hint
end

return {
    cached_is_nvim = cached_is_nvim,
    cached_is_windows = cached_is_windows,
    g = g,
    validate = validate
}
