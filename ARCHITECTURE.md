# Architecture
* UI
  - vim script [./autoload/linearf/ui/](./autoload/linearf/ui/)
    * every man knows his own business best
* Source
  - rust [./core/src/lib.rs](./core/src/lib.rs)
    * for heavy external resource acquisition, e.g. files and grep
  - vim script [./auto/linearf/source/](./auto/linearf/source/)
    * information that vim has, such as buffers
    * for vimmers who want to write vim script
* Match and Sort
  - it is a good use of rust [./core/src/lib.rs](./core/src/lib.rs)
* Action
  - vim script [./autoload/linearf/ui/actions/](./autoload/linearf/ui/actions/)
  - one of the UI
