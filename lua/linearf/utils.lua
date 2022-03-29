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

function M.is_mac()
    if C._is_mac == nil then C._is_mac = M.has('mac') end
    return C._is_mac
end

-- vim.g

function M.eval(s)
    if M.is_nvim() then
        return vim.api.nvim_eval(s)
    else
        return vim.eval(s)
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

function M.command_(...)
    if M.is_nvim() then
        return vim.api.nvim_command(string.format(...))
    else
        return vim.command(string.format(...))
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

function M.interval(ms, f)
    local options = M.dict({['repeat'] = -1})
    return vim.fn.timer_start(ms, f, options)
end

function M.augroup(name, xs)
    M.command("augroup " .. name)
    M.command("au!")
    for _, x in ipairs(xs) do M.command(x) end
    M.command("augroup END")
end

function M.win_id2tabwin(winid)
    if M.is_nvim() then
        return vim.fn.win_id2tabwin(winid)
    else
        local ret = {}
        for x in vim.fn.win_id2tabwin(winid)() do table.insert(ret, x) end
        return ret
    end
end

function M.setbufline(b, lnum, lines)
    return vim.fn.setbufline(b, lnum, M.list(lines))
end

function M.win_id2bufnr(winid)
    local infos = vim.fn.getwininfo(winid)
    return (infos[0] or infos[1])['bufnr']
end

-- PRIVATE

function M.lua_ver()
    if jit and jit.version ~= nil then return 'luajit' end
    local v = _VERSION
    local i = v:find(' ')
    local l = v:sub(i + 1, #v):gsub('[^%d]', '')
    return 'lua' .. l
end

return M
