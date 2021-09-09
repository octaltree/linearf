# Architecture
## Introduction
There are many fuzzy finders in vim: CtrlP, LeaderF, denite, telescope, clap, fzf, skim, fzf-preview, candle, etc. However, there were nothing that that had the feel of vim and the speed of native fzf, so I decided to make one.

I want to write in another language because vim script is slow, but I have to consider the cost of moving data to vim. After considering rpc, dynamic libraries, and wasm for speed and vim compability, I chose to load dynamic libraries written by rust via lua.

## UI [../autoload/linearf.vim](../autoload/linearf.vim)
Every man knows his own business best. All UI for view, selection, and action is done in vim script.

## Sessions [../model/core/src/session.rs](../model/core/src/session.rs)
denite has the ability to resume the last selection, which is so useful that the preview is not needed. List by source and matcher is named session and kept in memory as cache.

## Registration [../model/registrar/](../model/registrar/)
Sources and matchers will be provided as a crate so that rust assets can be used. Just before the build, [preprocessor](../model/registrar/preprocessor/main.rs) will add them to [registrar](../model/registrar/) accoding to the recipe.
