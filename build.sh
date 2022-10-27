#!/bin/bash

set -e

cd "$(dirname "$0")"

echo "Building the workspace packages..."

(set -x; cargo build)
(set -x; cargo test --no-run)
(set -x; cargo bench --no-run)

# We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
# It's also a little faster. If you wish to use the local version instead, swap out the below line.
# scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"
scrypto="scrypto"

echo "Building scrypto packages used in tests..."
(
    find "radix-engine/tests" -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -n1 -I '{}' bash -c "set -x; $scrypto build --path {}"
)
(
    find "simulator/tests" -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -n1 -I '{}' bash -c "set -x; $scrypto build --path {}"
)

echo "Building assets and examples..."
(
    find "assets" -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -n1 -I '{}' bash -c "set -x; $scrypto build --path {}"
)
(
    find "examples" -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
     # We use a slightly different formulation here so scrypto build picks up the `rust-toolchain` file and compiles with nightly
    | xargs -n1 -I '{}' bash -c "set -x; cd '{}'; $scrypto build" 
)
