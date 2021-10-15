local List = {}

local utils = require('linearf').utils

local NAME = 'linearf-vanilla-list'

function List.new()
    local this = {}
    this.winid = nil
    return setmetatable(this, {__index = List})
end

function List.flow(self, ctx, flow)
    self:open()

    -- print(session)
    -- local f = function()
    -- end
    -- local opt = {}
    -- opt['repeat'] = -1
    -- opt = utils.dict(opt)
    -- self.timer = vim.fn.timer_start(self.refresh_interval, f, opt)
end

function List.bufnr(self)
    local nr = vim.fn.bufnr(NAME)
    if nr == -1 then
        local new = vim.fn.bufadd(NAME)
        -- vim.fn.bufload(new)
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

function List.open(self)
    local bufnr = self:bufnr()
    if is_shown(bufnr) then return end
    if self.winid then self:close() end
    utils.command_("silent keepalt botright split buffer %d", bufnr)
    self.winid = vim.fn.win_getid()
end

function List.close(self)
    local tmp = vim.fn.win_getid()
    if vim.fn.win_gotoid(self.winid) == 1 then utils.command("silent close") end
    self.winid = nil
    vim.fn.win_gotoid(tmp)
end

function List.set_active(self)
    vim.fn.win_gotoid(self.winid)
end

return List
