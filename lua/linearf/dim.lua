local Dim = {}

function Dim.new()
    return setmetatable({}, {__index = Dim})
end

function Dim.from(t)
    return setmetatable(t, {__index = Dim})
end

function Dim.set(self, ...)
    local args = {...};
    if #args < 2 then return end
    local tmp = self
    for i = 1, #args - 2 do
        local k = args[i]
        if type(tmp[k]) ~= 'table' then tmp[k] = {} end
        tmp = tmp[k]
    end
    local k = args[#args - 1]
    local v = args[#args]
    tmp[k] = v
    return self
end

function Dim.get(self, ...)
    local args = {...};
    local ret = self
    for _, k in ipairs(args) do ret = (ret or {})[k] end
    return ret
end

return Dim
