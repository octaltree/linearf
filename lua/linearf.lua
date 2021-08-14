local M = {}

local path = require('linearf.path')
local bridge
local foo =2 

function M.start()
    path.append_bridge()
    bridge = require('bridge')
    foo = 3
    bridge.spawn()
end


local function is_array(t)
  local i = 0
  for _ in pairs(t) do
      i = i + 1
      if t[i] == nil then return false end
  end
  return true
end

function M.send(xs)
  print(vim.inspect(xs))
  --print(xs[1])
  --print(xs)
  --print(foo)
  --print(is_array(xs))
  --local ret = {}
  --for i=1,200000 do
  --  table.insert(ret, i)
  --end
  --bridge.send(xs)
  --return ret
end

local builder = {}

function M.new()
  builder = {}
end

function M.append(x)
  table.insert(builder, x)
end

function M.build(x)
  local ret = builder
  builder = {}
  return ret
end

return M
