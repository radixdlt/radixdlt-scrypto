#!/bin/bash

#set -x
set -e

cd "$(dirname "$0")"

# clean this worksapce
cargo clean

# clean this and other workspaces folders
(
    find "." -mindepth 2 -maxdepth 4 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "echo cleaning '{}'; cd '{}'; cargo clean"
)

# clean assets/blueprints
(
    find "assets/blueprints" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "echo cleaning '{}'; cd '{}'; cargo clean"
)

# clean examples
(
    find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "echo cleaning '{}'; cd '{}'; cargo clean"
)
