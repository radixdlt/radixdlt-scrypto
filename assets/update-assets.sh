#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Building packages..."
(cd account; scrypto build)
(cd sys-faucet; scrypto build)

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
  -o ./sys_faucet.wasm \
  ./sys-faucet/target/wasm32-unknown-unknown/release/sys_faucet.wasm
cp \
  ./sys-faucet/target/wasm32-unknown-unknown/release/sys_faucet.abi \
  ./sys_faucet.abi

echo "Done!"
