#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

cargo build --release

wasm-opt -Os -g \
--strip-debug --strip-dwarf --strip-producers \
-o ../../simulator/src/account.wasm \
target/wasm32-unknown-unknown/release/account.wasm

echo "Done!"