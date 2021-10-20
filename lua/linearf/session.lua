local Session = {}

function Session.new(id, senario_builder)
    local this = {}
    this.id = id
    this.senario_builder = senario_builder
    this.flow = {}
    return setmetatable(this, {__index = Session})
end

function Session.insert(self, fid, flow)
    self.flow[fid] = flow
    return self
end

return Session
