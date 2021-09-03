local M = {}
local Stack = {}

local stack

function Stack.new()
    local obj = {}
    obj.buf = {}
    setmetatable(obj, {__index = Stack})
    return obj
end

function Stack.push(self, x)
    table.insert(self.buf, x)
    return self
end

function Stack.pop(self) return table.remove(self.buf) end

function Stack.back(self) return self.buf[#self.buf] end

function Stack.slice_back(self, n)
    local ret = {unpack(self.buf, #self.buf - n + 1, #self.buf)}
    for _ = 1, n do table.remove(self.buf) end
    return ret
end

function M.new() stack = Stack.new() end

function M.push(x) stack:push(x) end

function M.dict() stack:push({}) end

function M.entry()
    local value = stack:pop()
    local key = stack:pop()
    local dict = stack:back()
    dict[key] = x
end

function M.array_finish(n) stack:push(stack:slice_back(n)) end

function M.finish()
    local ret = stack:pop()
    stack = Stack.new()
    return ret
end

return M
