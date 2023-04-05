#!/bin/bash

set -e

cd "$(dirname "$0")"

echo "Building the workspace packages (with all extended features)..."

(set -x; cargo build)
(set -x; cargo test --no-run)
(set -x; cargo bench --no-run)

echo "Building the engine in different configurations..."

(set -x; cd radix-engine; cargo build --features wasmer,resource_tracker)
(set -x; cd radix-engine; cargo build --no-default-features --features alloc)

echo "Building the simulator packages..."

(set -x; cd simulator; cargo build)
(set -x; cd simulator; cargo test --no-run)

# We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
# It's also a little faster. If you wish to use the local version instead, swap out the below line.
# scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"
scrypto="scrypto"

echo "Building scrypto packages used in tests..."
(
    find "radix-engine-tests/tests/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; $scrypto build --path {}"
)
(
    find "simulator/tests" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; $scrypto build --path {}"
)

echo "Building assets and examples..."
(
    find "assets/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; $scrypto build --path {}"
)
# Note - We use a slightly different formulation for the scrypto build line so that scrypto build picks up the `rust-toolchain` file and compiles with nightly
# This is possibly a rustup bug where it doesn't look for the toolchain file correctly (https://rust-lang.github.io/rustup/overrides.html) when using the `--manifest-path` flag
(
    find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; cd '{}'; $scrypto build" 
)
