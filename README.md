# Overview

`serverlooter` (WIP title ofc) is an incremental "hacking" game. See [docs](docs/) for further details.

The game is in active development, and has lots of quirks. Issues + pull requests are not welcome at this time, but you're welcome to fork and make any changes you like.

# Building

`./scripts/build-wasm.sh`

# Local Testing

You must build the wasm prior to serving the files. See [Building](#building).

`npx http-server wasm_out -o --p 1330`

# Attributions

- audio effects courtesy of [KenneyNL's All-in-One bundle](https://kenney.itch.io/kenney-game-assets)