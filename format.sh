#!/bin/bash

set -e

cd "$(dirname "$0")"

# Format all main package crates
(set -x; cargo fmt)

# Format the radix-clis crate
(set -x; cd radix-clis; cargo fmt)

(
    find "assets/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo fmt --manifest-path {}"
)
(
    find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo fmt --manifest-path {}"
)
(
    find "radix-engine-tests/assets/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo fmt --manifest-path {}"
)
(
    find "radix-clis/tests" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo fmt --manifest-path {}"
)

echo "All packages have been formatted."
