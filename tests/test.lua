local M = {}

local linearf = require('linearf')

function M.run()
    linearf.recipe.sources = {
        {
            name = "rustdoc",
            path = "flavors_rustdoc::Rustdoc"
        }
    }
    linearf.recipe.matchers = {
        {
            name = "substring",
            path = "flavors_plain::Substring"
        }
    }
    linearf.build()
    linearf.init(require('linearf-vanilla').new())
    linearf.senarios = {
        test = {
            linearf = {
                source = 'rustdoc',
                matcher = 'substring'
            }
        }
    }
    linearf.run('test')
end

return M
