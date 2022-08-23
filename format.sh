#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

(cd sbor; cargo fmt)
(cd sbor-derive; cargo fmt)
(cd sbor-tests; cargo fmt)
(cd scrypto; cargo fmt)
(cd scrypto-abi; cargo fmt)
(cd scrypto-derive; cargo fmt)
(cd scrypto-tests; cargo fmt)
(cd scrypto-unit; cargo fmt)
(cd radix-engine; cargo fmt)
(cd radix-engine-stores; cargo fmt)
(cd simulator; cargo fmt)
(cd transaction; cargo fmt)

scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"

(cd assets/account; $scrypto fmt)
(cd assets/system; $scrypto fmt)

(cd examples;
    find . -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -n1 -I '{}' $scrypto fmt --path {}
)
(cd radix-engine/tests;
    find . -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -n1 -I '{}' $scrypto fmt --path {}
)

echo "All packages have been formatted."
