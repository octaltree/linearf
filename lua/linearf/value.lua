local M = {}
local Buf = {}

function Buf.new()
    local obj = {}
    obj.buf = {}
    setmetatable(obj, {__index = Buf})
    return obj
end

function Buf.push(self, x)
    table.insert(self.buf, x)
    return self
end

function Buf.pop(self) return table.remove(self.buf) end

function Buf.back(self) return self.buf[#self.buf] end

function Buf.slice_back(self, n)
    local ret = {unpack(self.buf, #self.buf - n + 1, #self.buf)}
    for _ = 1, n do table.remove(self.buf) end
    return ret
end

function M.new() return Buf.new() end

function M.push(self, x) self:push(x) end

function M.dict(self) self:push({}) end

function M.entry(self)
    local value = self:pop()
    local key = self:pop()
    local dict = self:back()
    dict[key] = value
end

function M.array_finish(self, n) self:push(self:slice_back(n)) end

return M
