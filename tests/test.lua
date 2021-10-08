local M = {}

local linearf = require('linearf')

function M.build_blank()
    linearf.recipe.sources = {}
    linearf.recipe.matchers = {}
    linearf.recipe.converters = {}
    linearf.build()
end

function M.run()
    linearf.recipe.sources = {
        {
            name = "simple",
            path = "test_sources::source::Simple"
        },
        {
          name = "osstr",
          path = "test_sources::source::OsStr"
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
        simple = {
            linearf = {
                source = 'simple',
                matcher = 'substring'
            }
        },
        osstr = {
            linearf = {
                source = 'osstr',
                matcher = 'substring'
            }
        },
    }
    linearf.run('simple')
end

return M
