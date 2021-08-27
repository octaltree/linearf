local C = {}
local M = {cache = C}

function M.has(name) return vim.fn.has(name) == 1 end

function C.is_nvim()
    if C._is_nvim == nil then C._is_nvim = M.has('nvim') end
    return C._is_nvim
end

function C.is_windows()
    if C._is_windows == nil then C._is_windows = M.has('win32') end
    return C._is_windows
end

function M.g(name)
    if C.is_nvim() then
        return vim.g[name]
    else
        return vim.eval('g:' .. name)
    end
end

function M.call(f, ...) return vim.call(f, ...) end

return M
