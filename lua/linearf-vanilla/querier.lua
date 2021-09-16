local Querier = {}

local utils = require('linearf.utils')

local NAME = 'linearf-vanilla-querier'

function Querier.new()
    return setmetatable({}, {__index = Querier})
end

function Querier.start(self, session)
end

function Querier.open(self)
    local bufnr = vim.fn.bufadd(NAME)
    utils.command(string.format("silent keepalt aboveleft split buffer %d",
                                bufnr))
end

function Querier.close(self)
end

function Querier.set_active(self)
    vim.fn.win_gotoid(self.win_id)
end

return Querier
