local Session = {}

function Session.new(bridge, id, senario, senario_builder)
    local this = {}
    this.bridge = bridge
    this.id = id
    this.senario = senario
    this.senario_builder = senario_builder
    this.flow_id = nil
    this.on_flow_changed = nil
    return setmetatable(this, {__index = Session})
end

function Session.tick(self, senario)
    self.bridge.tick(self.id, senario)
    -- flow
end

function Session.query(self, query)
    local senario = self.senario_builder:build()
    if type(senario.linearf) ~= 'table' then senario.linearf = {} end
    senario.linearf.query = query
    self:tick(senario)
end

return Session
