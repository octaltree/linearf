local SenarioBuilder = {}

local function merge(a, b)
    local a_is_dict = type(a) == 'table' and #a == 0
    local b_is_dict = type(b) == 'table' and #b == 0
    if not a_is_dict or not b_is_dict then
        if b ~= nil then
            return b
        else
            return a
        end
    end
    if not a_is_dict or not b_is_dict then return b end
    local ret = {}
    for k, v in pairs(a) do ret[k] = v end
    for k, v in pairs(b) do ret[k] = merge(ret[k], v) end
    return ret
end

function SenarioBuilder.new(base, context_manager, diff)
    local this = {}
    this.base = base
    this.context_manager = context_manager
    this.diff = diff
    this.merge = merge
    return setmetatable(this, {__index = SenarioBuilder})
end

function SenarioBuilder.build(self)
    local ctx
    ctx = self.context_manager()
    if type(ctx) ~= 'table' then ctx = {} end
    return self.merge(self.merge(self.base, ctx), self.diff)
end

return SenarioBuilder
