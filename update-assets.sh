#!/bin/bash

set -x
set -e

# We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
# It's also a little faster. If you wish to use the local version instead, swap out the below line.
scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"
# scrypto="scrypto"

cd "$(dirname "$0")/assets/blueprints"

for crate_name in "faucet" "radiswap" "flash_loan" "genesis_helper" "metadata" "test_environment" "global_n_owned"
do
  echo "Building $crate_name..."
  (cd $crate_name; $scrypto build)

  cp \
    ./target/wasm32-unknown-unknown/release/$crate_name.{wasm,rpd} \
    ../
  echo "Done $crate_name!"
done
