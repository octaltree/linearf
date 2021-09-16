local Preview = {}

function Preview.new()
    return setmetatable({}, {__index = Preview})
end

function Preview.start(self, session)
end

function Preview.open(self)
end

function Preview.close(self)
end

return Preview
