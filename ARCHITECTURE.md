# Architecture
* UI
  - vim script [./autoload/linearfinder/ui/](./autoload/linearfinder/ui/)
    * every man knows his own business best
* Source
  - rust [./core/src/lib.rs](./core/src/lib.rs)
    * for heavy external resource acquisition, e.g. files and grep
  - vim script [./auto/linearfinder/source/](./auto/linearfinder/source/)
    * information that vim has, such as buffers
    * for vimmers who want to write vim script
* Match and Sort
  - it is a good use of rust [./core/src/lib.rs](./core/src/lib.rs)
* Action
  - vim script [./autoload/linearfinder/ui/actions/](./autoload/linearfinder/ui/actions/)
  - one of the UI
* Bridge between vim script and rust
  - nvim's lua and vim's if_lua [./lua/linearfinder/rpc.lua](./lua/linearfinder/rpc.lua)
