#!/usr/bin/env bash

set -e

export RUSTFLAGS=

cargo +nightly build --target wasm32-unknown-unknown --release
wasm-gc target/wasm32-unknown-unknown/release/wad_render.wasm -o wad_render.gc.wasm
