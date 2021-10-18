local linearf = require('linearf')
local utils = linearf.utils

-- https://github.com/Shougo/denite.nvim/blob/master/autoload/denite/filter.vim s:init_buffer
-- https://github.com/Shougo/denite.nvim/blob/master/rplugin/python3/denite/ui/default.py _init_buffer
local function win_common()
    utils.command('setlocal nobuflisted') -- vsplit makes listed enable
    utils.command('setlocal noswapfile')
    utils.command('setlocal bufhidden=hide')
    utils.command('setlocal buftype=nofile')
    utils.command('setlocal nocursorcolumn')
    utils.command('setlocal winfixheight')
    utils.command('setlocal foldcolumn=0')
    utils.command('setlocal nofoldenable')
    utils.command('setlocal nomodeline')
    utils.command('setlocal norelativenumber')
    utils.command('setlocal nospell')
    utils.command('setlocal noswapfile')
    utils.command('setlocal nowrap')
    utils.command('setlocal signcolumn=no') -- prompt?
    utils.command('setlocal nonumber')
end

local function setlocal_querier_win(senario)
    win_common()
    utils.command('setlocal nocursorline')
    utils.command('resize 1')
    -- vim.fn.setbufline(1, senario.linearf.query or '')

    utils.augroup('linearf_querier', {
        "au TextChanged,TextChangedI,TextchangedP <buffer> lua linearf.view:_query()"
    })
end

local function setlocal_list_win(senario)
    local params = senario.view
    win_common()
    utils.command_('setlocal %scursorline', params.cursorline and '' or 'no')
    -- utils.command('setlocal readonly')

    utils.augroup('linearf_list', {
        "autocmd CursorMoved <buffer> lua linearf.view.curline = vim.fn.line('.')"
    })
end

local function buffer(name)
    local nr = vim.fn.bufnr(name)
    if nr == -1 then
        local new = vim.fn.bufadd(name)
        -- utils.command(new .. ' buffer')
        vim.fn.bufload(new)
        return new
    end
    return nr
end

local function is_shown(bufnr)
    if utils.is_nvim() then
        for _, x in ipairs(vim.fn.tabpagebuflist()) do
            if x == bufnr then return true end
        end
    else
        for x in vim.fn.tabpagebuflist()() do
            if x == bufnr then return true end
        end
    end
    return false
end

local function all(f, xs)
    local ret = true
    for _, x in ipairs(xs) do if not f(x) then ret = false end end
    return ret
end

local function values(dic)
    local ret = {}
    for _, v in pairs(dic) do table.insert(ret, v) end
    return ret
end

return function(Vanilla)
    function Vanilla._close_all(self)
        local tmp = vim.fn.win_getid()
        if vim.fn.win_gotoid(self.querier_win) == 1 then
            utils.command("silent close")
        end
        if vim.fn.win_gotoid(self.list_win) == 1 then
            utils.command("silent close")
        end
        self.querier_win = nil
        self.list_win = nil
        vim.fn.win_gotoid(tmp)
    end

    function Vanilla._ensure_open(self, flow, buff)
        if all(is_shown, values(buff)) then return end
        if self.querier_win or self.list_win then
            self:_close_all()
            self.querier_win = nil
            self.list_win = nil
        end
        utils.command('silent keepalt botright sb ' .. self.LIST)
        self.list_win = vim.fn.win_getid()
        setlocal_list_win(flow.senario)
        utils.command('silent keepalt aboveleft sb ' .. self.QUERIER)
        self.querier_win = vim.fn.win_getid()
        setlocal_querier_win(flow.senario)
    end

    function Vanilla._delete_all_buffers(self)
    end

    function Vanilla._ensure_bufexists(self)
        local l = buffer(self.LIST)
        local q = buffer(self.QUERIER)
        return {list = l, querier = q}
    end

    function Vanilla._query(self)
        local q = vim.fn.getline(1)
        linearf.query(self.current.session_id, q)
    end
end