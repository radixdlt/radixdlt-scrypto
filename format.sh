#!/bin/bash

set -e

cd "$(dirname "$0")"

# Format all main package crates
(set -x; cargo fmt)

# Format the simulator crate
(set -x; cd simulator; cargo fmt)

(
    find "assets/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo fmt --manifest-path {}"
)
(
    find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo fmt --manifest-path {}"
)
(
    find "radix-engine-tests/tests/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo fmt --manifest-path {}"
)
(
    find "simulator/tests" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo fmt --manifest-path {}"
)

echo "All packages have been formatted."
