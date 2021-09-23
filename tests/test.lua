local M = {}

local linearf = require('linearf')

function M.run()
    linearf.recipe = {
        crates = {
            {
                name = "rustdoc",
                dep = [[{ git = "https://github.com/octaltree/linearf-my-flavors", branch = "dev" }]]
            },
            {
                name = "plain",
                dep = [[{ git = "https://github.com/octaltree/linearf-my-flavors", branch = "dev" }]]
            }
        },
        sources = {
            {
                name = "rustdoc",
                path = "rustdoc::Rustdoc"
            }
        },
        matchers = {
            {
                name = "substring",
                path = "plain::Substring"
            }
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
