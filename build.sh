#!/bin/bash

set -e

cd "$(dirname "$0")"

# This should align with format.sh, format-check.sh, check.sh, test.sh, clean.sh, update-cargo-locks-minimally.sh
echo "Building the workspace packages and tests (with all extended features)..."

(set -x; cargo build; cargo test --no-run; cargo bench --no-run)
(set -x; cargo build -p radix-engine-profiling --all-features; cargo test -p radix-engine-profiling --all-features --no-run; cargo bench -p radix-engine-profiling --all-features --no-run)
(set -x; cargo build --features fuzzing)

echo "Building scrypto packages and tests using cargo build, to catch errors quickly..."

(set -x; cd radix-engine-tests/assets/blueprints; cargo build; cargo test --no-run)
(set -x; cd radix-clis/tests/blueprints; cargo build; cargo test --no-run)
(set -x; cd scrypto-test/tests/blueprints; cargo build; cargo test --no-run)
(set -x; cd scrypto-test/assets/blueprints; cargo build; cargo test --no-run)
(set -x; cd radix-transaction-scenarios/assets/blueprints; cargo build; cargo test --no-run)
(set -x; cd scrypto-compiler/tests/assets/scenario_1; cargo build; cargo test --no-run)
(set -x; cd scrypto-compiler/tests/assets/scenario_2; cargo build; cargo test --no-run)

echo "Building the engine in different configurations..."

(set -x; cd radix-engine; cargo build --no-default-features --features alloc,lru)
(set -x; cd radix-engine; cargo build --features resource_tracker)

# We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
# It's also a little faster. If you wish to use the local version instead, swap out the below line.
# scrypto="cargo run --manifest-path $PWD/radix-clis/Cargo.toml --bin scrypto $@ --"
scrypto="scrypto"

echo "Building scrypto packages used in tests with scrypto build..."
(
    find "radix-engine-tests/assets/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; $scrypto build --path {}"
)
(
    find "radix-clis/tests/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; $scrypto build --path {}"
)
(
    find "scrypto-test/tests/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; $scrypto build --path {}"
)
(
    find "scrypto-test/assets/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; $scrypto build --path {}"
)
(
    find "radix-transaction-scenarios/assets/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; $scrypto build --path {}"
)

echo "Building examples..."
# Note - We use a slightly different formulation for the scrypto build line so that scrypto build picks up the `rust-toolchain` file and compiles with nightly
# This is possibly a rustup bug where it doesn't look for the toolchain file correctly (https://rust-lang.github.io/rustup/overrides.html) when using the `--manifest-path` flag
(
    find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; cd '{}'; $scrypto build"
)

# We don't rebuild `radix-engine/assets` because they are fixed at genesis/the relevant protocol update, and they no longer compile
