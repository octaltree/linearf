local M = {cache = {}}

function M.has(name) return vim.fn.has(name) == 1 end

function M.cache.is_nvim()
    if cache._is_nvim == nil then cache._is_nvim = M.has('nvim') end
    return cache.is_nvim
end

function M.cache.is_windows()
    if cache._is_windows == nil then cache._is_windows = M.has('win32') end
    return cache._is_windows
end

function g(name)
    if cache.is_nvim() then
        return vim.g[name]
    else
        return vim.eval('g:' .. name)
    end
end

function M.validate(table)
    if cache.is_nvim() then return vim.validate(table) end
    -- Use only as a type hint
end

return M
