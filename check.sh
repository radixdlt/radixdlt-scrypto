#!/bin/bash

set -e

cd "$(dirname "$0")"

(cd sbor; cargo fmt --check --quiet)
(cd sbor-derive; cargo fmt --check --quiet)
(cd sbor-tests; cargo fmt --check --quiet)
(cd scrypto; cargo fmt --check --quiet)
(cd scrypto-abi; cargo fmt --check --quiet)
(cd scrypto-derive; cargo fmt --check --quiet)
(cd scrypto-tests; cargo fmt --check --quiet)
(cd radix-engine; cargo fmt --check --quiet)
(cd simulator; cargo fmt --check --quiet)
(cd transaction-manifest; cargo fmt --check --quiet)

(cd assets/account; cargo fmt --check --quiet)
(cd assets/system; cargo fmt --check --quiet)
(cd examples; find . -maxdepth 1 -type d \( ! -name . \) -exec bash -c "cd '{}' && cargo fmt --check --quiet" \;)
(cd radix-engine/tests; find . -maxdepth 1 -type d \( ! -name . \) -exec bash -c "cd '{}' && cargo fmt --check --quiet" \;)

echo "All packages have been formatted."
