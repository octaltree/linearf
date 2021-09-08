# Architecture
## Introduction
TODO


* UI
  * [../autoload/linearf/ui.vim](../autoload/linearf/ui.vim)
    - every man knows his own business best
    - View
    - Selection
    - Action
* Action
  - [../autoload/linearf/ui/actions/](../autoload/linearf/ui/actions/)
  - one of the UI
* Source
  - [../model/core/src/source.rs](../core/src/lib.rs)
    * for heavy external resource acquisition, e.g. files and grep
  - [../autoload/linearf/source/](../autoload/linearf/source/)
    * information that vim has, such as buffers
    * for vimmers who want to write vim script
* Match and Sort
  - it is a good use of rust [../core/src/lib.rs](../core/src/lib.rs)
