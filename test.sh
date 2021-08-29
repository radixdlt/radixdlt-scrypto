#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Building examples..."
(cd blueprints/account; cargo build --release)
(cd examples/helloworld; cargo build --release)
(cd examples/no_std; cargo build --release)
(cd examples/gumball-machine; cargo build --release)

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

echo "Testing with std..."
(cd sbor; cargo test)
(cd sbor-derive; cargo test)
(cd sbor-tests; cargo test)
(cd scrypto; cargo test)
(cd scrypto-derive; cargo test)
(cd scrypto-tests; cargo test)
(cd scrypto-types; cargo test)
(cd radix-engine; cargo test)
(cd simulator; bash ./tests/run.sh)

echo "Testing with no_std..."
(cd sbor; cargo test --no-default-features --features alloc)
(cd sbor-tests; cargo test --no-default-features --features alloc)
(cd scrypto; cargo test --no-default-features --features alloc)
(cd scrypto-abi; cargo test --no-default-features --features alloc)
(cd scrypto-types; cargo test --no-default-features --features alloc)
(cd scrypto-tests; cargo test --no-default-features --features alloc)
(cd radix-engine; cargo test --no-default-features --features alloc)

echo "Congrats! All tests passed."
