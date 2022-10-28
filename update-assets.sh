#!/bin/bash

set -x
set -e

# We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
# It's also a little faster. If you wish to use the local version instead, swap out the below line.
# scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"
scrypto="scrypto"

cd "$(dirname "$0")/assets/blueprints"

echo "Building packages..."
(cd account; $scrypto build)
(cd faucet; $scrypto build)

echo "Publishing artifacts..."
npx wasm-opt@1.3 \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ../account.wasm \
  ./account/target/wasm32-unknown-unknown/release/account.wasm
cp \
  ./account/target/wasm32-unknown-unknown/release/account.abi \
  ../account.abi

npx wasm-opt@1.3 \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ../faucet.wasm \
  ./faucet/target/wasm32-unknown-unknown/release/faucet.wasm
cp \
  ./faucet/target/wasm32-unknown-unknown/release/faucet.abi \
  ../faucet.abi

echo "Done!"
