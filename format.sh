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
(cd scrypto-types; cargo fmt)
(cd radix-engine; cargo fmt)
(cd examples/helloworld; cargo fmt)
(cd examples/no_std; cargo fmt)

echo "All packages have been formatted."
