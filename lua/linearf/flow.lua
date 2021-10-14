local Flow = {}

function Flow.new(bridge, sid, fid, senario)
    local this = {}
    this.bridge = bridge
    this.session_id = sid
    senario.source = nil
    senario.matcher = nil
    this.senario = senario
    this.flow_id = fid
    return setmetatable(this, {__index = Flow})
end

function Flow.status(self)
    return self.bridge.flow_status(self.session_id, self.flow_id)
end

function Flow.items(self, ge, lt)
    return self.bridge.flow_items(self.session_id, self.flow_id, ge, lt)
end

return Flow