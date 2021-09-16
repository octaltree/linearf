local Vanilla = {}

local utils = require('linearf.utils')

function Vanilla.new()
    local this = {}
    this.list = require('linearf-vanilla.list').new()
    this.querier = require('linearf-vanilla.querier').new()
    this.preview = require('linearf-vanilla.preview').new()
    this.refresh_interval = 15
    this.querier_on_start = 'inactive' -- 'inactive'/'active'/'insert'
    this.deactivate_querier_on_normal = true
    return setmetatable(this, {__index = Vanilla})
end

function Vanilla.start(self, session)
    self.list:start(session)
    self.querier:start(session)
    self.preview:start(session)
    if self.querier_on_start == 'insert' then
        self.querier:set_insert()
    elseif self.querier.querier_on_start == 'active' then
        self.querier:set_active()
    else
        self.list:set_active()
    end
end

function Vanilla.close(self)
    self.list:close()
    self.querier:close()
    self.preview:close()
end

-- start, open, close, switch_active, switch_deactive, update?

return Vanilla
