#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Building packages..."
(cd account; cargo build --target wasm32-unknown-unknown --release)
(cd system; cargo build --target wasm32-unknown-unknown --release)

echo "Publishing artifacts..."
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./account.wasm \
  ./account/target/wasm32-unknown-unknown/release/account.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./system.wasm \
  ./system/target/wasm32-unknown-unknown/release/system.wasm

echo "Done!"
