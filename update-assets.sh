#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Building packages..."
(cd blueprints/account; cargo build --release)
(cd examples/helloworld; cargo build --release)
(cd examples/no_std; cargo build --release)
(cd examples/gumball-machine; cargo build --release)
(cd examples/vendor; cargo build --release)

echo "Publishing assets..."
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./assets/account.wasm \
  ./blueprints/account/target/wasm32-unknown-unknown/release/account.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./assets/helloworld.wasm \
  ./examples/helloworld/target/wasm32-unknown-unknown/release/helloworld.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./assets/no_std.wasm \
  ./examples/no_std/target/wasm32-unknown-unknown/release/no_std.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./assets/gumball-machine.wasm \
  ./examples/gumball-machine/target/wasm32-unknown-unknown/release/gumball-machine.wasm
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./assets/vendor.wasm \
  ./examples/vendor/target/wasm32-unknown-unknown/release/vendor.wasm
echo "Done!"
