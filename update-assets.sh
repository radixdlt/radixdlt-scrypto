#!/bin/bash

set -x
set -e

# We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
# It's also a little faster. If you wish to use the local version instead, swap out the below line.
scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"
# scrypto="scrypto"

cd "$(dirname "$0")/assets/blueprints"

# See `publish_package_1mib` for how to produce the right sized wasm
$scrypto build --disable-wasm-opt --path ../../radix-engine-tests/tests/blueprints/large_package
cp ../../radix-engine-tests/tests/blueprints/target/wasm32-unknown-unknown/release/large_package.{wasm,rpd} ..
ls -al ../large_package.*

for crate_name in "faucet" "radiswap" "flash_loan" "genesis_helper" "metadata" "test_environment" "global_n_owned" "kv_store"
do
  echo "Building $crate_name..."
  (cd $crate_name; $scrypto build)

  cp \
    ./target/wasm32-unknown-unknown/release/$crate_name.{wasm,rpd} \
    ../
  echo "Done $crate_name!"
done