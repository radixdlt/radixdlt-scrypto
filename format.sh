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
(cd examples/nft/magic-card; scrypto fmt)
(cd examples/nft/sporting-event; scrypto fmt)
(cd examples/defi/auto-lend; scrypto fmt)
(cd examples/defi/mutual-farm; scrypto fmt)
(cd examples/defi/price-oracle; scrypto fmt)
(cd examples/defi/radiswap; scrypto fmt)
(cd examples/defi/regulated-token; scrypto fmt)
(cd examples/defi/synthetics; scrypto fmt)
(cd examples/defi/x-perp-futures; scrypto fmt)

echo "All packages have been formatted."
