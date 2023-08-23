#!/bin/bash

set -x
set -e

# We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
# It's also a little faster. If you wish to use the local version instead, swap out the below line.
scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"
# scrypto="scrypto"

cd "$(dirname "$0")/assets/blueprints"


echo "Building metadata..."
(cd metadata; $scrypto build)
npx wasm-opt@1.3 \
  -Os -g \
  --strip-debug --strip-dwarf --strip-producers \
  -o ../metadata.wasm \
  ./target/wasm32-unknown-unknown/release/metadata.wasm
cp \
  ./target/wasm32-unknown-unknown/release/metadata.rpd \
  ../metadata.rpd

echo "Done!"
