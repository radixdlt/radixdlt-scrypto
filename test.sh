#!/bin/bash

set -e

cd "$(dirname "$0")"

echo "Testing with std"
(cd sbor; cargo test)
(cd sbor-derive; cargo test)
(cd sbor-tests; cargo test)
(cd scrypto; cargo test)
(cd scrypto-derive; cargo test)
(cd scrypto-tests; cargo test)

echo "Testing with no_std"
(cd sbor; cargo test --no-default-features --features json,alloc)
(cd sbor-tests; cargo test --no-default-features --features alloc)
(cd scrypto; cargo test --no-default-features --features alloc)
(cd scrypto-tests; cargo test --no-default-features --features alloc)

echo "Congrats! All tests passed."
