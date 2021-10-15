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
    this.last_flow = nil
    this.line = {}
    return setmetatable(this, {__index = Vanilla})
end

Vanilla.DEFAULT = {refresh_interval = 15, cursorline = true}

-- senario view params
-- this.refresh_interval = 15
-- this.querier_on_start = 'inactive' -- 'inactive'/'active'/'insert'
-- this.deactivate_querier_on_normal = true

function Vanilla.flow(self, ctx, flow)
    local buff = self:_ensure_bufexists()
    local done = self:_write_first_view(ctx, flow, buff)
    self:_ensure_open(ctx, flow, buff)
    utils.command('redraw')
    if not done then self:_start_incremental(ctx, flow) end
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

function Vanilla._write_first_view(self, ctx, flow, buff)
    local n = flow.senario.linearf.first_view
    local items
    do
        local r = flow:items(0, n)
        if not r.ok then return false end
        items = r.value
    end
    local lines = {}
    for _, item in ipairs(items) do table.insert(lines, item.view) end
    vim.fn.setbufline(buff.list, 1, lines)
    vim.fn.deletebufline(buff.list, #lines + 1, '$')
    return flow:status():map(function(t)
        return t.done and t.count <= n
    end):unwrap_or(false)
end

function Vanilla._start_incremental(self, ctx, flow)
end

-- open preview manually
-- hide only preview

-- function Vanilla.start(self, session)
--    self.list:start(session)
--    self.querier:start(session)
--    self.preview:start(session)
--    if self.querier_on_start == 'insert' then
--        self.querier:set_insert()
--    elseif self.querier.querier_on_start == 'active' then
--        self.querier:set_active()
--    else
--        self.list:set_active()
--    end
-- end

-- start, open, close, switch_active, switch_deactive, update?

return Vanilla
