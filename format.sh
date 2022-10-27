#!/bin/bash

set -e

cd "$(dirname "$0")"

# Format all main package crates
(set -x; cargo fmt)

# We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
# It's also a little faster. If you wish to use the local version instead, swap out the below line.
# scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"
scrypto="scrypto"

(
    find "assets" -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -n1 -I '{}' bash -c "set -x; $scrypto fmt --path {}"
)
(
    find "examples" -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -n1 -I '{}' bash -c "set -x; $scrypto fmt --path {}"
)
(
    find "radix-engine/tests" -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -n1 -I '{}' bash -c "set -x; $scrypto fmt --path {}"
)
(
    find "simulator/tests" -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -n1 -I '{}' bash -c "set -x; $scrypto fmt --path {}"
)

echo "All packages have been formatted."
