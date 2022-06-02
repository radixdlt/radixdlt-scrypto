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

(cd assets/account; scrypto fmt --check --quiet)
(cd assets/system; scrypto fmt --check --quiet)
(cd examples; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' scrypto fmt --path {} --check --quiet)
(cd radix-engine/tests; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' scrypto fmt --path {} --check --quiet)

echo "Code format check passed!"
