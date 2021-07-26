#!/bin/bash

set -e

cd "$(dirname "$0")"

echo "Testing with std"
(cd scrypto; cargo test)
(cd scrypto-tests; cargo test)

echo "Testing with no_std"
(cd scrypto; cargo test --no-default-features --features alloc)
(cd scrypto-tests; cargo test --no-default-features --features alloc)
