local Session = {}

function Session.new(id, scenario_builder)
    local this = {}
    this.id = id
    this.scenario_builder = scenario_builder
    this.flow = {}
    return setmetatable(this, {__index = Session})
end

function Session.insert(self, fid, flow)
    self.flow[fid] = flow
    return self
end

return Session
