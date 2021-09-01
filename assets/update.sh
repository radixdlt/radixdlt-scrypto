#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Publishing assets..."
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./account.wasm \
  ../blueprints/account/target/wasm32-unknown-unknown/release/account.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./helloworld.wasm \
  ../examples/helloworld/target/wasm32-unknown-unknown/release/helloworld.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./no_std.wasm \
  ../examples/no_std/target/wasm32-unknown-unknown/release/no_std.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./gumball-machine.wasm \
  ../examples/gumball-machine/target/wasm32-unknown-unknown/release/gumball-machine.wasm
echo "Done!"
