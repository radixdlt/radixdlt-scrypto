#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Testing with std..."
(cd sbor; cargo test)
(cd sbor-derive; cargo test)
(cd sbor-tests; cargo test)
(cd scrypto; cargo test)
(cd scrypto-derive; cargo test)
(cd scrypto-tests; cargo test)
(cd radix-engine; cargo test)

echo "Testing with no_std..."
(cd sbor; cargo test --no-default-features --features alloc)
(cd sbor-tests; cargo test --no-default-features --features alloc)
(cd scrypto; cargo test --no-default-features --features alloc)
(cd scrypto-abi; cargo test --no-default-features --features alloc)
(cd scrypto-tests; cargo test --no-default-features --features alloc)
(cd radix-engine; cargo test --no-default-features --features alloc)

echo "Building examples..."
(cd assets/account; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd assets/system; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd examples/core/cross-blueprint-call; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd examples/core/flat-admin; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd examples/core/gumball-machine; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd examples/core/hello-nft; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd examples/core/hello-world; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd examples/core/managed-access; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd examples/core/no-std-lib; cargo build --target wasm32-unknown-unknown --release)

echo "Running simulator..."
(cd simulator; bash ./tests/resim.sh)
(cd simulator; bash ./tests/scrypto.sh)

echo "Running benchmark..."
(cd sbor-tests; cargo bench)
(cd radix-engine; cargo bench)

echo "Congrats! All tests passed."
