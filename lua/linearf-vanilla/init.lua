local Vanilla = {}
require('linearf-vanilla.win')(Vanilla)

local utils = require('linearf').utils

-- REQUIRED

function Vanilla.new()
    local this = {}
    this.LIST = 'linearf-vanilla-list'
    this.QUERIER = 'linearf-vanilla-querier'
    this.list_win = nil
    this.querier_win = nil

    this.prev_flow = nil
    this.current = nil

    this.curline = 1
    this.line = {}
    return setmetatable(this, {__index = Vanilla})
end

Vanilla.DEFAULT = {
    refresh_interval = 15,
    cursorline = true,
    chunk_size = 7000,

    rendering = {first = 100, before = 100, after = 200, last = 100}
}

-- senario view params
-- this.querier_on_start = 'inactive' -- 'inactive'/'active'/'insert'
-- this.deactivate_querier_on_normal = true

function Vanilla.flow(self, ctx, flow)
    -- utils.command("let g:_linearf_time = reltime()")
    self:_save_prev_flow(flow)
    -- TODO: resume curline
    local buff = self:_ensure_bufexists()
    local done = self:_write_first_view(flow, buff)
    self:_ensure_open(flow, buff)
    utils.command('redraw')
    utils.command("echomsg 'first view' .. reltimestr(reltime(g:_linearf_time))")
    if not done then self:_start_incremental(flow, buff) end
end

function Vanilla.destruct(self)
    self:hide_all()
    self:_delete_all_buffers()
end

-- DUCK TYPING

function Vanilla.hide_all(self)
    self:_close_all()
end

-- PRIVATE

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

function Vanilla._write_first_view(self, flow, buff)
    local n = flow.senario.linearf.first_view
    local items = flow:items({{0, n}}, {id = true, view = true}):unwrap()[1]
    local lines = {}
    for _, item in ipairs(items) do table.insert(lines, item.view) end
    vim.fn.setbufline(buff.list, 1, lines)
    -- vim.fn.deletebufline(buff.list, #lines + 1, '$') -- TODO: slow
    return flow:status():map(function(t)
        return t.done and t.count <= n
    end):unwrap_or(false)
end

local function calc_ranges(cur, len, rendering)
    local first = {0, rendering.first}
    local around = {cur - rendering.before, cur + rendering.after + 1}
    local last = {len - rendering.last, len}
    local a = around[1] <= first[2]
    local b = last[1] <= around[2]
    if a and b then
        return {{first[1], last[2]}}
    elseif a then
        return {{first[2], around[2]}, last}
    elseif b then
        return {first, {around[1], last[2]}}
    else
        return {first, around, last}
    end
end

local function empty(n)
    local s = ''
    local ret = {}
    for _ = 1, n do table.insert(ret, s) end
    return ret
end

function Vanilla._start_incremental(self, flow, buff)
    local params = flow.senario.view
    local count = 0
    local first = true
    utils.interval(params.refresh_interval, function(timer)
        if self.current ~= flow then
            vim.fn.timer_stop(timer)
            return
        end
        local status
        do
            local r = flow:status()
            if not r.ok then return end
            status = r.value
        end
        if status.done then
            vim.fn.timer_stop(timer)
            self:_write_last_view(flow, buff, status.count)
            return
        end
        if count == status.count then
            return
        else
            count = status.count
        end
        local ranges = calc_ranges(self.curline - 1, count, params.rendering)
        local range_items
        do
            local r = flow:items(ranges, {id = true, view = true})
            if not r.ok then return end
            range_items = r.value
        end
        for i = 1, #range_items do
            local lines = {}
            for _, item in ipairs(range_items[i]) do
                table.insert(lines, item.view)
            end
            if self.current ~= flow then
                vim.fn.timer_stop(timer)
                return
            end
            vim.fn.setbufline(buff.list, ranges[i][1] + 1, lines)
        end
        if first then
            utils.command(
                "echomsg 'tmp first line' .. reltimestr(reltime(g:_linearf_time))")
            first = false
        end
    end)
end

function Vanilla._write_last_view(self, flow, buff, count)
    local l = 1
    local chunk = flow.senario.view.chunk_size
    utils.interval(0, function(timer)
        local items
        do
            local r = flow:items({{l, l + chunk}}, {id = true, view = true})
            if not r.ok then
                vim.fn.timer_stop(timer)
                return
            end
            items = r.value[1]
        end
        local lines = {}
        for _, item in ipairs(items) do table.insert(lines, item.view) end
        if self.current ~= flow then
            vim.fn.timer_stop(timer)
            return
        end
        vim.fn.setbufline(buff.list, l, lines)
        if l == 1 then
            utils.command(
                "echomsg 'last first line' .. reltimestr(reltime(g:_linearf_time))")
        end
        l = l + chunk
        if l > count then
            utils.command(
                "echomsg 'last done' .. reltimestr(reltime(g:_linearf_time))")
            vim.fn.timer_stop(timer)
        end
    end)
end

-- open preview manually
-- hide only preview

-- start, open, close, switch_active, switch_deactive, update?

return Vanilla
