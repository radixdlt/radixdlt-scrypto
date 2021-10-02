#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Building packages..."
(cd assets/account; cargo build --target wasm32-unknown-unknown --release)
(cd assets/system; cargo build --target wasm32-unknown-unknown --release)

echo "Publishing artifacts..."
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./assets/account.wasm \
  ./assets/account/target/wasm32-unknown-unknown/release/out.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./assets/system.wasm \
  ./assets/system/target/wasm32-unknown-unknown/release/out.wasm

echo "Done!"
