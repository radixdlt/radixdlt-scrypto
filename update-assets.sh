#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/assets"

echo "Building packages..."
(cd account; scrypto build)
(cd faucet; scrypto build)

echo "Publishing artifacts..."
wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./account.wasm \
  ./account/target/wasm32-unknown-unknown/release/account.wasm
cp \
  ./account/target/wasm32-unknown-unknown/release/account.abi \
  ./account.abi

wasm-opt \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ./faucet.wasm \
  ./faucet/target/wasm32-unknown-unknown/release/faucet.wasm
cp \
  ./faucet/target/wasm32-unknown-unknown/release/faucet.abi \
  ./faucet.abi

echo "Done!"
