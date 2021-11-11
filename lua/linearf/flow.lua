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

function Flow.items(self, ranges, fields)
    return self.bridge.flow_items(self.session_id, self.flow_id, ranges, fields)
end

function Flow.id_items(self, ids, fields)
    return self.bridge.flow_id_items(self.session_id, self.flow_id, ids, fields)
end

function Flow.query(self, q)
    local linearf = require('linearf')
    return linearf._query(self.session_id, q)
end

return Flow
