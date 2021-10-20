local Vanilla = {}

local linearf = require('linearf')
local utils = linearf.utils

local function time(name)
    if not linearf._debug then return end
    utils.command_("echomsg '%s' .. reltimestr(reltime(g:_linearf_time))", name)
end

do -- REQUIRED
    function Vanilla.new()
        local this = {}
        this.list_win = nil
        this.querier_win = nil

        this.prev_flow = nil
        this.current = nil

        this.curline = 1
        this.line = {}
        return setmetatable(this, {__index = Vanilla})
    end

    Vanilla.QUERIER = 'linearf-vanilla-querier'
    Vanilla.DEFAULT = {
        cursorline = true,
        querier_on_start = 'inactive', -- 'inactive'|'active'|'insert'

        refresh_interval = 15,
        view_size = 3000,
        chunk_size = 6000
    }

    -- ctx = {
    --     awake = 'session'|'flow'|'resume
    -- }
    function Vanilla.flow(self, ctx, flow)
        ctx.refresh = ctx.awake == 'session' or ctx.awake == 'resume'
        self:_save_prev_flow(flow)
        local buff, done, skip = self:_first_view(ctx, flow)
        self:_ensure_open(ctx, flow.senario, buff, skip)
        if ctx.awake == 'session' then
            self:_set_cursor(flow.senario)
            utils.command("redraw")
        end
        time('first view')
        if not done then self:_start_incremental(flow, buff) end
    end

    function Vanilla.destruct(self)
        self:hide_all()
    end
end

do -- DUCK TYPING
    function Vanilla.hide_all(self)
        self:_close_all()
    end
end

do -- PRIVATE
    local FIELDS = {id = true, view = true}

    function Vanilla._save_prev_flow(self, flow)
        self.prev_flow = self.current
        self.current = flow
        if self.prev_flow then
            local last = self.prev_flow
            if not self.line[last.session_id] then
                self.line[last.session_id] = {}
            end
            self.line[last.session_id][last.flow_id] = self.curline
            -- winsaveview() ?
        end
    end

    local nofile = {}
    do
        function nofile.delete(nr)
            utils.command('bwipeout ' .. nr)
        end

        local function _create(name)
            local new = vim.fn.bufadd(name)
            vim.fn.setbufvar(new, '&buftype', 'nofile')
            vim.fn.setbufvar(new, '&buflisted', 0)
            vim.fn.setbufvar(new, '&swapfile', 0)
            vim.fn.setbufvar(new, '&bufhidden', 'wipe')
            vim.fn.setbufvar(new, '&modeline', 0)
            vim.fn.bufload(new)
            return new
        end

        local function find(name)
            return vim.fn.bufnr(string.format('^%s$', name))
        end

        function nofile.new(name)
            local nr = find(name)
            if nr ~= -1 then nofile.delete(nr) end
            return _create(name)
        end

        function nofile.named(name)
            local nr = find(name)
            if nr ~= -1 then return nr end
            return _create(name)
        end
    end

    function Vanilla._first_view(self, ctx, flow)
        local vars = flow.senario.linearf
        local buff = {list = {}}
        buff.querier = nofile.named(self.QUERIER)
        if ctx.refresh then
            vim.fn.setbufline(buff.querier, 1, flow.senario.linearf.query)
        end
        local size = vars.first_view
        local items, done, count, last
        do
            local r = flow:items({{0, size}}, FIELDS)
            if not r.ok then return buff, false, false end
            done = r.value.done
            count = r.value.count
            items = r.value[1]
            last = done and count <= size
        end
        local name = string.format("%s_%s%s", vars.query, count,
                                   last and '' or '+')
        buff.list[1] = nofile.new(name)
        local lines = {}
        for _, item in ipairs(items) do table.insert(lines, item.view) end
        vim.fn.setbufline(buff.list[1], 1, lines)
        return buff, last, count == 0 and not last
    end

    local function all(f, xs)
        local ret = true
        for _, x in ipairs(xs) do if not f(x) then ret = false end end
        return ret
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

    local function win_exists(winid)
        return utils.win_id2tabwin(winid)[1] ~= 0
    end

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

    -- https://github.com/Shougo/denite.nvim/blob/master/autoload/denite/filter.vim s:init_buffer
    -- https://github.com/Shougo/denite.nvim/blob/master/rplugin/python3/denite/ui/default.py _init_buffer
    local function win_common()
        utils.command('setlocal buftype=nofile')
        utils.command('setlocal nobuflisted') -- vsplit makes listed enable
        utils.command('setlocal noswapfile')
        utils.command('setlocal bufhidden=wipe')
        utils.command('setlocal nomodeline')

        utils.command('setlocal nocursorcolumn')
        utils.command('setlocal winfixheight')
        utils.command('setlocal foldcolumn=0')
        utils.command('setlocal nofoldenable')
        utils.command('setlocal norelativenumber')
        utils.command('setlocal nospell')
        utils.command('setlocal nowrap')
        utils.command('setlocal signcolumn=no') -- prompt?
        utils.command('setlocal nonumber')
    end

    local function setlocal_querier_win(ctx, senario)
        win_common()
        utils.command('setlocal nocursorline')
        utils.command('resize 1')
        -- vim.fn.setbufline(1, senario.linearf.query or '')

        local first_changedtick = ctx.refresh
        linearf.view._querier_on_changed = function()
            if first_changedtick then
                first_changedtick = false
                return
            end
            linearf.query(linearf.view.current.session_id, vim.fn.getline(1))
        end
        utils.augroup('linearf_querier', {
            "au TextChanged,TextChangedI,TextchangedP <buffer> lua linearf.view._querier_on_changed()"
        })
    end

    local function setlocal_list_win(senario)
        local params = senario.view
        win_common()
        utils.command_('setlocal %scursorline', params.cursorline and '' or 'no')
        -- utils.command('setlocal readonly')

        -- winsave
        -- utils.augroup('linearf_list', {
        --    "autocmd CursorMoved <buffer> lua linearf.view.curline = vim.fn.line('.')"
        -- })
    end

    local function current_tab()
        return utils.eval("win_id2tabwin(win_getid())[0]")
    end

    function Vanilla._ensure_open(self, ctx, senario, buff, skip)
        local list = buff.list[1]
        local querier = buff.querier
        local function set_buffer()
            if not skip then
                vim.fn.win_gotoid(self.list_win)
                utils.command("buffer " .. list)
                setlocal_list_win(senario)
            end
            vim.fn.win_gotoid(self.querier_win)
            utils.command("buffer " .. querier)
            setlocal_querier_win(ctx, senario)
        end
        if ctx.refresh then
            local tab = current_tab()
            local win_shown = function(w)
                return utils.win_id2tabwin(w)[1] == tab
            end
            if self.querier_win and self.list_win and
                all(win_shown, {self.querier_win, self.list_win}) then
                set_buffer()
                return
            end
        else
            if all(win_exists, {self.querier_win, self.list_win}) then
                set_buffer()
                return
            end
        end
        if self.querier_win or self.list_win then
            self:_close_all()
            self.querier_win = nil
            self.list_win = nil
        end
        utils.command('silent keepalt botright sb ' .. list)
        self.list_win = vim.fn.win_getid()
        setlocal_list_win(senario)
        utils.command('silent keepalt aboveleft sb ' .. querier)
        self.querier_win = vim.fn.win_getid()
        setlocal_querier_win(ctx, senario)
    end

    function Vanilla._set_cursor(self, senario)
        local status = senario.view.querier_on_start
        if status == 'active' or status == 'insert' then
            vim.fn.win_gotoid(self.querier_win)
        else
            vim.fn.win_gotoid(self.list_win)
        end
        if status == 'insert' then utils.command("startinsert!") end
    end

    function Vanilla._start_incremental(self, flow, buff)
        local senario = flow.senario
        local open = function(b)
            local cur = vim.fn.win_getid()
            if cur ~= self.list_win and cur ~= self.querier_win then
                return
            end
            if vim.fn.win_gotoid(self.list_win) ~= 1 then return end
            local page = vim.fn.winsaveview()
            utils.command('buffer ' .. b)
            vim.fn.winrestview(page)
            setlocal_list_win(senario)
            vim.fn.win_gotoid(cur)
        end
        local pre = nil
        utils.interval(senario.view.refresh_interval, function(timer)
            if self.current ~= flow then
                vim.fn.timer_stop(timer)
                return
            end
            local items, done, count
            do
                local r = flow:items({{0, senario.view.view_size}}, FIELDS)
                if not r.ok then return end
                done = r.value.done
                count = r.value.count
                items = r.value[1]
            end
            if count == 0 and not done then
                -- skip flicker
                return
            end
            if pre == count and not done then
                return
            else
                pre = count
            end
            local name = string.format("%s_%s%s", senario.linearf.query, count, done and '' or '+')
            local lines = {}
            for _, item in ipairs(items) do table.insert(lines, item.view) end
            do -- write
                local b = nofile.named(name)
                vim.fn.setbufline(b, 1, lines)
                table.insert(buff.list, b)
                if self.current ~= flow then
                    vim.fn.timer_stop(timer)
                    return
                end
                open(b)
            end
            if done then
                time('tmp done')
                vim.fn.timer_stop(timer)
                self:_write_last_view(flow, buff, #items, count)
                return
            end
        end)
    end

    function Vanilla._write_last_view(self, flow, buff, offset, count)
        local l = offset + 1
        local chunk = flow.senario.view.chunk_size
        local b = buff.list[#buff.list]
        utils.interval(0, function(timer)
            if self.current ~= flow then
                vim.fn.timer_stop(timer)
                return
            end
            local items
            do
                local r = flow:items({{l - 1, l - 1 + chunk}}, FIELDS)
                if not r.ok then
                    vim.fn.timer_stop(timer)
                    return
                end
                items = r.value[1]
            end
            local lines = {}
            for _, item in ipairs(items) do table.insert(lines, item.view) end
            vim.fn.setbufline(b, l, lines)
            l = l + #items
            if l > count then
                time('last done')
                vim.fn.timer_stop(timer)
            end
        end)
    end
end

return Vanilla
-- vim: foldnestmax=2 shiftwidth=4
