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
(cd radix-engine; cargo fmt)
(cd simulator; cargo fmt)
(cd transaction-manifest; cargo fmt)

(cd assets/account; cargo fmt)
(cd assets/system; cargo fmt)
(cd examples; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' cargo fmt --manifest-path {}/Cargo.toml)
(cd radix-engine/tests; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' cargo fmt --manifest-path {}/Cargo.toml)

echo "All packages have been formatted."
