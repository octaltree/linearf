local SenarioBuilder = {}

-- Senario:
--   * linearf: LinearfVars,
--   * source: SourceParams,
--   * matcher: MatcherParams,
--   * view: ViewParams,
local DEFAULT = {
    linearf = {
        query = '',
        converters = {},
        cache_sec = 60,
        cache_across_sessions = true,
        chunk_size = 32767
    },
    source = {},
    matcher = {},
    view = {}
}

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

local function foldl(f, x, xs)
    local ret = x
    for _, y in ipairs(xs) do ret = f(ret, y) end
    return ret
end

function SenarioBuilder.build(self)
    local ctx
    ctx = self.context_manager()
    if type(ctx) ~= 'table' then ctx = {} end
    return foldl(self.merge, DEFAULT, {self.base, ctx, self.diff})
end

return SenarioBuilder
