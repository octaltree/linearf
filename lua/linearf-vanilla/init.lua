local M = {}

local utils = require('linearf.utils')

M.new = function()
    return setmetatable({time = 15}, {__index = M})
end

M.start = function(self, session)
    print(session)
    local f = function()
    end
    local rep = {}
    rep['repeat'] = -1
    rep = utils.dict(rep)
    self.timer = vim.fn.timer_start(self.time, f, rep)
end

return M
