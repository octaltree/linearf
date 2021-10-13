local C = {}
local M = {cache = C}

M.unpack = table.unpack or unpack

function M.has(name)
    return vim.fn.has(name) == 1
end

function M.is_nvim()
    if C._is_nvim == nil then C._is_nvim = M.has('nvim') end
    return C._is_nvim
end

function M.is_windows()
    if C._is_windows == nil then C._is_windows = M.has('win32') end
    return C._is_windows
end

function M.g(name)
    if M.is_nvim() then
        return vim.g[name]
    else
        return vim.eval('g:' .. name)
    end
end

function M.call(f, ...)
    return vim.call(f, ...)
end

function M.command(s)
    if M.is_nvim() then
        return vim.api.nvim_command(s)
    else
        return vim.command(s)
    end
end

function M.dict(x)
    if M.is_nvim() then
        return x
    else
        return vim.dict(x)
    end
end

function M.list(x)
    if M.is_nvim() then
        return x
    else
        return vim.list(x)
    end
end

function M.value(x)
    if M.is_nvim() then return x end
    if type(x) == 'table' then
        if #x > 0 then
            return vim.list(x)
        else
            return vim.dict(x)
        end
    end
    return x
end

function M.readdir(...)
    if M.is_nvim() then return vim.fn.readdir(...) end
    -- :h lua-list
    local ret = {}
    for x in vim.fn.readdir(...)() do table.insert(ret, x) end
    return ret
end

-- PRIVATE

function M.lua_ver()
    if jit and jit.version ~= nil then return 'luajit' end
    local v = _VERSION
    local i = v:find(' ')
    local l = v:sub(i + 1, #v):gsub('[^%d]', '')
    return 'lua' .. l
end

-- function M.echo_error(s)
--     local msg = '[linearf] ' .. s
--     local quoted = vim.fn.string(msg)
--     M.command(string.format("echohl Error | echomsg %s | echohl None", quoted))
-- end

return M
