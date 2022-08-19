#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Building packages..."
(cd account; cargo build --target wasm32-unknown-unknown --release)
(cd sys-faucet; cargo build --target wasm32-unknown-unknown --release)
(cd sys-utils; cargo build --target wasm32-unknown-unknown --release)

echo "Publishing artifacts..."
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./account.wasm \
  ./account/target/wasm32-unknown-unknown/release/account.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./sys_faucet.wasm \
  ./sys-faucet/target/wasm32-unknown-unknown/release/sys_faucet.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./sys_utils.wasm \
  ./sys-utils/target/wasm32-unknown-unknown/release/sys_utils.wasm

echo "Done!"
