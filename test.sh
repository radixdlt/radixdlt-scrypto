#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Testing with std"
(cd sbor; cargo test)
(cd sbor-derive; cargo test)
(cd sbor-tests; cargo test)
(cd scrypto; cargo test)
(cd scrypto-derive; cargo test)
(cd scrypto-tests; cargo test)
(cd scrypto-types; cargo test)
(cd radix-engine; cargo test)
(cd simulator; cargo run -- help)

echo "Testing with no_std"
(cd sbor; cargo test --no-default-features --features alloc)
(cd sbor-tests; cargo test --no-default-features --features alloc)
(cd scrypto; cargo test --no-default-features --features alloc)
(cd scrypto-abi; cargo test --no-default-features --features alloc)
(cd scrypto-types; cargo test --no-default-features --features alloc)
(cd scrypto-tests; cargo test --no-default-features --features alloc)

echo "Testing examples"
(cd examples/helloworld; cargo build --target wasm32-unknown-unknown --release)
(cd examples/no_std; cargo build --target wasm32-unknown-unknown  --release)

echo "Congrats! All tests passed."
