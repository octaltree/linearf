local Preview = {}

function Preview.new()
    return setmetatable({}, {__index = Preview})
end

function Preview.flow(self, ctx, flow)
end

function Preview.open(self)
end

function Preview.close(self)
end

return Preview
