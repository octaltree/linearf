local C = {}
local M = {cache = C}

function M.sep() return '/' end

function M.join(parts) return table.concat(parts, M.sep()) end

function C.root()
  return '/home/octaltree/workspace/linearf'
end

local name = 'lib?.so'
local release = M.join {C.root(), 'bridge', 'target', 'release', name}
local debug = M.join {C.root(), 'bridge', 'target', 'debug', name}
package.cpath = table.concat({package.cpath, release, debug}, ';')

local bridge = require('bridge')
bridge.spawn()
