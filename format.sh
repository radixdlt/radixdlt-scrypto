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

(cd assets/account; scrypto fmt)
(cd assets/system; scrypto fmt)
(cd examples; find . -maxdepth 1 -type d \( ! -name . \) -exec bash -c "cd '{}' && scrypto fmt" \;)
(cd radix-engine/tests; find . -maxdepth 1 -type d \( ! -name . \) -exec bash -c "cd '{}' && scrypto fmt" \;)

echo "All packages have been formatted."
