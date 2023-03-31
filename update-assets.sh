#!/bin/bash

set -x
set -e

# We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
# It's also a little faster. If you wish to use the local version instead, swap out the below line.
# scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"
scrypto="scrypto"

cd "$(dirname "$0")/assets/blueprints"

echo "Building faucet..."
(cd faucet; $scrypto build)
npx wasm-opt@1.3 \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ../faucet.wasm \
  ./target/wasm32-unknown-unknown/release/faucet.wasm
cp \
  ./target/wasm32-unknown-unknown/release/faucet.schema \
  ../faucet.schema

echo "Building radiswap..."
(cd radiswap; $scrypto build)
npx wasm-opt@1.3 \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ../radiswap.wasm \
  ./target/wasm32-unknown-unknown/release/radiswap.wasm
cp \
  ./target/wasm32-unknown-unknown/release/radiswap.schema \
  ../radiswap.schema

echo "Building flash_loan..."
(cd flash_loan; $scrypto build)
npx wasm-opt@1.3 \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ../flash_loan.wasm \
  ./target/wasm32-unknown-unknown/release/flash_loan.wasm
cp \
  ./target/wasm32-unknown-unknown/release/flash_loan.schema \
  ../flash_loan.schema

echo "Done!"
