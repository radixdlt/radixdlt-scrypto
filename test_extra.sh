#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Testing scrypto with release profile..."
(cd sbor; cargo test --release)

echo "Testing raidx engine with wasmer..."
(cd radix-engine; cargo test --features wasmer)

echo "Testing crates with no_std..."
(cd sbor; cargo test --no-default-features --features alloc)
(cd sbor-tests; cargo test --no-default-features --features alloc)
(cd scrypto; cargo test --no-default-features --features alloc,prelude)
(cd scrypto-abi; cargo test --no-default-features --features alloc)
(cd scrypto-tests; cargo test --no-default-features --features alloc)
(cd radix-engine; cargo test --no-default-features --features alloc)

echo "Congrats! All extra tests passed."
