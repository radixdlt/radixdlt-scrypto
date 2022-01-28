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
(cd examples/core/cross-blueprint-call; scrypto fmt)
(cd examples/core/flat-admin; scrypto fmt)
(cd examples/core/gumball-machine; scrypto fmt)
(cd examples/core/hello-nft; scrypto fmt)
(cd examples/core/hello-world; scrypto fmt)
(cd examples/core/managed-access; scrypto fmt)
(cd examples/core/no-std-lib; scrypto fmt)

echo "All packages have been formatted."
