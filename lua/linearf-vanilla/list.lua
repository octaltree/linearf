local List = {}

local utils = require('linearf.utils')

local NAME = 'linearf-vanilla-list'

function List.new()
    local this = {}
    return setmetatable(this, {__index = List})
end

function List.start(self, session)
    local bufnr = vim.fn.bufadd(NAME)
    utils.command(
        string.format("silent keepalt botright split buffer %d", bufnr))
    self.bufnr = vim.fn.bufnr('%')
    self.winid = vim.fn.win_getid()

    -- print(session)
    -- local f = function()
    -- end
    -- local opt = {}
    -- opt['repeat'] = -1
    -- opt = utils.dict(opt)
    -- self.timer = vim.fn.timer_start(self.refresh_interval, f, opt)
end

function List.open(self)
    -- silent execute a:context['filter_split_direction'] 'split' 'denite-filter'
    -- let g:denite#_filter_winid = win_getid()
    -- let g:denite#_filter_bufnr = bufnr('%')
end

function List.close(self)
end

function List.set_active(self)
end

return List
