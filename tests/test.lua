local M = {}

local linearf = require('linearf')

function M.run()
    linearf.recipe.sources = {
        {
            name = "simple",
            path = "test_sources::source::Simple"
        }
    }
    linearf.recipe.matchers = {
        {
            name = "substring",
            path = "test_sources::matcher::Substring"
        }
    }
    linearf.recipe.converters = {
        {
            name = "OddEven",
            path = "test_sources::converter::OddEven"
        }
    }
    linearf.build()
    local view = {
        start = function(self, session)
        end
    }
    linearf.init(view)
    linearf.senarios = {
        test = {
            linearf = {
                source = 'simple',
                matcher = 'substring'
            }
        }
    }
    linearf.run('test')
end

return M
