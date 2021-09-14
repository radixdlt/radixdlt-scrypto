#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Building packages..."
(cd examples/account; cargo build --target wasm32-unknown-unknown --release)
(cd examples/helloworld; cargo build --target wasm32-unknown-unknown --release)
(cd examples/no_std; cargo build --target wasm32-unknown-unknown --release)
(cd examples/gumball-machine; cargo build --target wasm32-unknown-unknown --release)
(cd examples/gumball-machine-vendor; cargo build --target wasm32-unknown-unknown --release)

echo "Publishing assets..."
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./assets/account.wasm \
  ./examples/account/target/wasm32-unknown-unknown/release/account.wasm
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
  -o ./assets/gumball-machine-vendor.wasm \
  ./examples/gumball-machine-vendor/target/wasm32-unknown-unknown/release/gumball-machine-vendor.wasm
echo "Done!"
