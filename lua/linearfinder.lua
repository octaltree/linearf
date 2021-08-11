local function start()
    package.cpath = package.cpath .. ';' .. '/home/octaltree/workspace/linearfinder/lua/linearfinder/bridge/target/release/lib?.so';
    local bridge = require('bridge')
    --print(bridge.sum(2, 3))
    --print('start')
    bridge.spawn()
end

return {start = start}
