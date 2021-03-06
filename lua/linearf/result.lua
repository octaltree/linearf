local Result = {}

function Result.Ok(x)
    return setmetatable({ok = true, value = x}, {__index = Result})
end

function Result.Err(e)
    return setmetatable({ok = false, value = e}, {__index = Result})
end

function Result.pcall(f, ...)
    local ok, value = pcall(f, ...)
    if ok then
        return Result.Ok(value)
    else
        return Result.Err(value)
    end
end

function Result.map(self, f)
    if not self.ok then return self end
    return Result.Ok(f(self.value))
end

function Result.map_err(self, f)
    if self.ok then return self end
    return Result.Err(f(self.value))
end

function Result.and_then(self, f)
    if not self.ok then return self end
    return f(self.value)
end

function Result.or_else(self, f)
    if self.ok then return self end
    return f(self.value)
end

function Result.unwrap(self)
    if self.ok then
        return self.value
    else
        error(self.value)
    end
end

function Result.unwrap_or(self, x)
    if self.ok then
        return self.value
    else
        return x
    end
end

return Result
