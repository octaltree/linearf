local Session = {}

function Session.new(bridge, id, senario, senario_builder)
    local this = {}
    this.bridge = bridge
    this.id = id
    senario.source = nil
    senario.matcher = nil
    this.senario = senario
    this.senario_builder = senario_builder
    this.flow_id = nil
    this.on_flow_changed = nil
    return setmetatable(this, {
        __index = Session
    })
end

function Session.tick(self, senario)
    local fid = self.bridge.tick(self.id, senario):unwrap()
    -- flow
end

function Session.query(self, query)
    local senario = self.senario_builder:build()
    if type(senario.linearf) ~= 'table' then senario.linearf = {} end
    senario.linearf.query = query
    self:tick(senario)
end

return Session