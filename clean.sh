#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

(cd sbor; cargo clean)
(cd sbor-derive; cargo clean)
(cd sbor-tests; cargo clean)
(cd scrypto; cargo clean)
(cd scrypto-derive; cargo clean)
(cd scrypto-tests; cargo clean)
(cd radix-engine; cargo clean)
(cd radix-engine-stores; cargo clean)
(cd transaction; cargo clean)
(cd simulator; cargo clean)

(cd assets/account; cargo clean)
(cd assets/sys-faucet; cargo clean)
(cd examples/hello-world; cargo clean)
(cd examples/no-std; cargo clean)
