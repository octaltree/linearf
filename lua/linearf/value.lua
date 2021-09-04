local Value = {}
local Buf = {}

function Buf.new()
    local obj = {}
    setmetatable(obj, {__index = Buf})
    return obj
end

function Buf.push(self, x)
    table.insert(self, x)
    return self
end

function Buf.pop(self) return table.remove(self) end

function Buf.back(self) return self[#self] end

function Buf.slice_back(self, n)
    local ret = {unpack(self, #self - n + 1, #self)}
    for _ = 1, n do table.remove(self) end
    return ret
end

function Value.new()
    local b = Buf.new()
    setmetatable(b, {__index = Value})
    return b
end

function Value.push(self, x) Buf.push(self, x) end

function Value.dict(self) Buf.push(self, {}) end

function Value.entry(self)
    local value = self:pop()
    local key = self:pop()
    local dict = self:back()
    dict[key] = value
end

function Value.array_finish(self, n) self:push(self:slice_back(n)) end

function Value.finish(self) return self end

return Value
