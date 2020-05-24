#!/usr/bin/env bash

set -e

export RUSTFLAGS=
export CARGO_INCREMENTAL=0

cargo build --target wasm32-unknown-unknown --release
wasm-gc target/wasm32-unknown-unknown/release/wad_render.wasm -o wad_render.gc.wasm
# cp target/wasm32-unknown-unknown/release/wad_render.wasm wad_render.gc.wasm

# cargo build --target wasm32-unknown-unknown
# wasm-gc target/wasm32-unknown-unknown/debug/wad_render.wasm -o wad_render.gc.wasm
# cp target/wasm32-unknown-unknown/debug/wad_render.wasm wad_render.gc.wasm
