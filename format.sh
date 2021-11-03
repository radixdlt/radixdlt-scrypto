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
(cd radix-engine/tests/everything; cargo fmt)
(cd assets/account; cargo fmt)
(cd assets/system; cargo fmt)
(cd examples/helloworld; cargo fmt)
(cd examples/features/no_std; cargo fmt)
(cd examples/features/cross-component-call; cargo fmt)
(cd simulator; cargo fmt)

echo "All packages have been formatted."
