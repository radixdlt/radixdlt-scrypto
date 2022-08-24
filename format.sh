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

# We use a globally loaded scrypto CLI so that this script works even if the code doesn't compile at present
# It's also a little faster. If you wish to use the local version instead, swap out the below line.
# scrypto="cargo run --manifest-path $PWD/simulator/Cargo.toml --bin scrypto $@ --"
scrypto="scrypto"

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
