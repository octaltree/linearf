local Vanilla = {}

local Dim = require('linearf.dim')
local linearf = require('linearf')
local utils = linearf.utils

local function time(name)
    if not linearf._debug then return end
    utils.command_("echomsg '%s' .. reltimestr(reltime(g:_linearf_time))", name)
end

local function initialize_global_map()
    local commands = function(cs)
        for _, c in ipairs(cs) do utils.command(c) end
    end
    commands {
        'nnor <Plug>(linearf-hide-all) :<c-u>lua linearf.view:hide_all()<CR>',
        'inor <Plug>(linearf-hide-all) <cmd>lua linearf.view:hide_all()<CR>',
        'nnor <Plug>(linearf-goto-list) :<c-u>lua linearf.view:goto_list()<CR>',
        'inor <Plug>(linearf-goto-list) <esc>:<c-u>lua linearf.view:goto_list()<CR>',
        'nnor <Plug>(linearf-goto-querier) :<c-u>lua linearf.view:goto_querier()<CR>',
        'inor <Plug>(linearf-goto-querier) <cmd>lua linearf.view:goto_querier()<CR>',
        'nnor <Plug>(linearf-goto-querier-insert) :<c-u>lua linearf.view:goto_querier_insert()<CR>',
        'inor <Plug>(linearf-goto-querier-insert) <cmd>lua linearf.view:goto_querier_insert()<CR>'
    }
end

do -- REQUIRED
    function Vanilla.new()
        local this = {}
        this.orig_win = nil
        this.list_win = nil
        this.querier_win = nil

        this.prev_flow = nil
        this.current = nil
        this.session_view = Dim.new()
        this.shown = {}

        initialize_global_map()
        return setmetatable(this, {__index = Vanilla})
    end

    Vanilla.QUERIER = 'linearf-vanilla-querier'
    Vanilla.DEFAULT = {
        cursorline = true,
        querier_on_start = 'inactive', -- 'inactive'|'active'|'insert'

        refresh_interval = 15,
        view_size = 1000,
        chunk_size = 6000
    }

    -- ctx = {
    --     awake = 'session'|'flow'|'resume
    -- }
    function Vanilla.flow(self, ctx, flow)
        ctx.refresh = ctx.awake == 'session' or ctx.awake == 'resume'
        self:_save_orig_win()
        self:_save_prev_flow(flow)
        local resume_view = self:_resume_view(ctx, flow)
        local buff, done, skip = self:_first_view(ctx, flow, resume_view)
        self:_ensure_open(ctx, flow, buff, skip)
        self:_set_cursor(ctx, flow, resume_view)
        time('first view')
        if not done then self:_start_incremental(flow, buff) end
    end

    function Vanilla.destruct(self)
        self:hide_all()
    end
end

do -- DUCK TYPING
    -- vanilla has only one set of windows at most across all tabs, so view_id is not needed
    function Vanilla.hide_all(self, _view_id)
        self:_close_all()
    end

    function Vanilla.goto_list(self, _view_id)
        vim.fn.win_gotoid(self.list_win)
    end

    function Vanilla.goto_querier(self, _view_id)
        vim.fn.win_gotoid(self.querier_win)
    end

    function Vanilla.goto_querier_insert(self, _view_id)
        if vim.fn.win_gotoid(self.querier_win) ~= 1 then return end
        vim.fn.feedkeys('A', 'n')
    end
end

do -- PRIVATE
    local FIELDS = {id = true, view = true}

    function Vanilla._save_orig_win(self)
        local current = vim.fn.win_getid()
        if self.querier_win == current or self.list_win == current then
            return
        end
        self.orig_win = current
    end

    function Vanilla._save_prev_flow(self, flow)
        self.prev_flow = self.current
        self.current = flow
    end

    function Vanilla._resume_view(self, ctx, flow)
        local resume_view
        do
            local v = self.session_view:get(flow.session_id, flow.flow_id)
            if ctx.awake == 'resume' and v then resume_view = v end
        end
        return resume_view
    end

    local nofile = {}
    do
        function nofile.delete(nr)
            utils.command('bwipeout ' .. nr)
        end

        local function _create(name, hidden)
            local new = vim.fn.bufadd(name)
            vim.fn.setbufvar(new, '&buftype', 'nofile')
            vim.fn.setbufvar(new, '&buflisted', 0)
            vim.fn.setbufvar(new, '&swapfile', 0)
            vim.fn.setbufvar(new, '&bufhidden', hidden)
            vim.fn.setbufvar(new, '&modeline', 0)
            vim.fn.bufload(new)
            return new
        end

        local function find(name)
            return vim.fn.bufnr(string.format('^%s$', name))
        end

        function nofile.new(name)
            -- local nr = find(name)
            -- if nr ~= -1 then nofile.delete(nr) end
            return _create(name, 'wipe')
        end

        function nofile.named(name)
            local nr = find(name)
            if nr ~= -1 then return nr end
            return _create(name, 'hide')
        end
    end

    local function title(query, count, source_count, done)
        return string.format('"%s" %s/%s%s', query, count, source_count,
                             done and '' or '+')
    end

    local function _first_view_size(flow, resume_view)
        local default = flow.senario.linearf.first_view
        return (resume_view or {}).lnum or default
    end

    function Vanilla._first_view(self, ctx, flow, resume_view)
        local vars = flow.senario.linearf
        local buff = {list = {}}
        buff.querier = nofile.named(self.QUERIER)
        if ctx.refresh then
            utils.setbufline(buff.querier, 1, {flow.senario.linearf.query})
        end
        local size = _first_view_size(flow, resume_view)
        local items, done, count, source_count, last
        do
            local r = flow:items({{0, size}}, FIELDS)
            if not r.ok then return buff, false, false end
            done = r.value.done
            count = r.value.count
            source_count = r.value.source_count
            items = r.value[1]
            last = done and count <= size
        end
        buff.list[1] = nofile.new(title(vars.query, count, source_count, last))
        local lines = {}
        for _, item in ipairs(items) do table.insert(lines, item.view) end
        utils.setbufline(buff.list[1], 1, lines)
        self.shown = items
        return buff, last, count == 0 and not last
    end

    local function all(f, xs)
        local ret = true
        for _, x in ipairs(xs) do if not f(x) then ret = false end end
        return ret
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

    local function function_hash(f)
        return tostring(f):gsub("^function: ", '')
    end

    function Vanilla._items(self)
        local cur = (self.session_view:get(self.current.session_id, self.current.flow_id) or {lnum = 1})['lnum']
        local xs = {self.shown[cur]}
        local ids = {}
        for _, x in ipairs(xs) do
            table.insert(ids, x.id)
        end
        local fields = {id = true, value = true, info = true}
        return self.current:id_items(ids, fields):unwrap()
    end

    function Vanilla._execute(self, dic_name, fh)
        local dic = self.current.senario.linearf[dic_name]
        local fn = nil
        for _, f in pairs(dic) do
            if function_hash(f) == fh then
                fn = f
            end
        end
        local items = self:_items()
        local tmp = vim.fn.win_getid()
        vim.fn.win_gotoid(self.orig_win)
        fn(items)
        vim.fn.win_gotoid(tmp)
    end

    local function setlocal_querier_win(ctx, flow)
        win_common()
        utils.command('setlocal ft=linearf-vanilla-querier')
        utils.command('setlocal nocursorline')
        utils.command('resize 1')

        -- TODO: args
        for k, v in pairs(flow.senario.linearf.querier_nnoremap) do
            local h = function_hash(v)
            local r = string.format(':<c-u>lua linearf.view:_execute("querier_nnoremap", %q)<CR>', h)
            utils.command(string.format('nnor <silent><buffer>%s %s', k, r))
        end
        for k, v in pairs(flow.senario.linearf.querier_inoremap) do
            local h = function_hash(v)
            local r = string.format('<cmd>lua linearf.view:_execute("querier_inoremap", %q)<CR>', h)
            utils.command(string.format('inor <silent><buffer>%s %s', k, r))
        end

        local first_changedtick = ctx.refresh
        linearf.view._querier_on_changed = function()
            if first_changedtick then
                first_changedtick = false
                return
            end
            linearf.view.current:query(vim.fn.getline(1))
        end
        utils.augroup('linearf_querier', {
            "au TextChanged,TextChangedI,TextchangedP <buffer> lua linearf.view._querier_on_changed()"
        })
    end

    local function setlocal_list_win(flow)
        local senario = flow.senario
        local params = senario.view
        win_common()
        utils.command('setlocal ft=linearf-vanilla-list')
        utils.command_('setlocal %scursorline', params.cursorline and '' or 'no')

        for k, v in pairs(flow.senario.linearf.list_nnoremap) do
            local h = function_hash(v)
            local r = string.format(':<c-u>lua linearf.view:_execute("list_nnoremap", %q)<CR>', h)
            utils.command(string.format('nnor <silent><buffer>%s %s', k, r))
        end

        -- linearf-toggle-select, select-all, unselect-all?

        utils.augroup('linearf_list', {
            string.format(
                "au CursorMoved <buffer> lua linearf.view.session_view:set(%s, %s, vim.fn.winsaveview())",
                flow.session_id, flow.flow_id)
        })
    end

    local function current_tab()
        return utils.eval("win_id2tabwin(win_getid())[0]")
    end

    function Vanilla._ensure_open(self, ctx, flow, buff, skip)
        local list = buff.list[1]
        local querier = buff.querier
        local function set_buffer()
            if list and not skip then
                vim.fn.win_gotoid(self.list_win)
                utils.command("buffer " .. list)
                setlocal_list_win(flow)
            end
            vim.fn.win_gotoid(self.querier_win)
            utils.command("buffer " .. querier)
            setlocal_querier_win(ctx, flow)
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
        setlocal_list_win(flow)
        utils.command('silent keepalt aboveleft sb ' .. querier)
        self.querier_win = vim.fn.win_getid()
        setlocal_querier_win(ctx, flow)
    end

    function Vanilla._set_cursor(self, ctx, flow, resume_view)
        local senario = flow.senario
        if ctx.awake == 'session' then
            local status = senario.view.querier_on_start
            if status == 'active' or status == 'insert' then
                vim.fn.win_gotoid(self.querier_win)
            else
                vim.fn.win_gotoid(self.list_win)
            end
            if status == 'insert' then
                -- utils.command("startinsert!") -- The drawing of the list is faulty.
                vim.fn.feedkeys('A', 'n')
            end
        elseif ctx.awake == 'resume' then
            vim.fn.win_gotoid(self.list_win)
            if resume_view then vim.fn.winrestview(resume_view) end
        end
    end

    function Vanilla._start_incremental(self, flow, buff)
        local senario = flow.senario
        local open = function(b)
            local cur = vim.fn.win_getid()
            if cur ~= self.list_win and cur ~= self.querier_win then
                return
            end
            -- if vim.fn.win_gotoid(self.list_win) ~= 1 then return end
            -- local page = vim.fn.winsaveview()
            -- utils.command('buffer ' .. b)
            -- vim.fn.winrestview(page)
            -- setlocal_list_win(flow)
            -- vim.fn.win_gotoid(cur)
            _G['_linearf_open'] = function()
                local page = vim.fn.winsaveview()
                utils.command('buffer ' .. b)
                vim.fn.winrestview(page)
                setlocal_list_win(flow)
            end
            local cur_nr = vim.fn.win_id2win(cur)
            local list_nr = vim.fn.win_id2win(self.list_win)
            utils.command_('%swindo lua _linearf_open()', list_nr)
            utils.command_('%swindo :', cur_nr)
            _G['_linearf_open'] = nil
        end
        local pre = nil
        local lazy = 10
        utils.interval(senario.view.refresh_interval, function(timer)
            if self.current ~= flow then
                vim.fn.timer_stop(timer)
                return
            end
            local items, done, count, source_count
            do
                local r = flow:items({{0, senario.view.view_size}}, FIELDS)
                if not r.ok then return end
                done = r.value.done
                count = r.value.count
                source_count = r.value.source_count
                items = r.value[1]
            end
            do -- skip flicker
                if lazy > 0 and count < senario.linearf.first_view and not done then
                    lazy = lazy - 1
                    return
                end
            end
            if pre == count and not done then
                return
            else
                pre = count
            end
            local lines = {}
            for _, item in ipairs(items) do
                table.insert(lines, item.view)
            end
            do -- write
                local b = nofile.new(title(senario.linearf.query, count,
                                           source_count, done))
                utils.setbufline(b, 1, lines)
                self.shown = items
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
            for _, item in ipairs(items) do
                table.insert(lines, item.view)
            end
            utils.setbufline(b, l, lines)
            for _, item in ipairs(items) do
                table.insert(self.shown, item)
            end
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
