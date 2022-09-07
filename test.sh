#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Testing with std..."
(cd sbor; cargo test)
(cd sbor-derive; cargo test)
(cd sbor-tests; cargo test)
(cd scrypto; cargo test)
(cd scrypto; cargo test --release)
(cd scrypto-derive; cargo test)
(cd scrypto-tests; cargo test)
(cd radix-engine; cargo test)
(cd radix-engine; cargo test --features wasmer)
(cd transaction; cargo test)

echo "Testing with no_std..."
(cd sbor; cargo test --no-default-features --features alloc)
(cd sbor-tests; cargo test --no-default-features --features alloc)
(cd scrypto; cargo test --no-default-features --features alloc,prelude)
(cd scrypto; cargo test --no-default-features --features alloc,prelude --release)
(cd scrypto-abi; cargo test --no-default-features --features alloc)
(cd scrypto-tests; cargo test --no-default-features --features alloc)

echo "Building system packages and examples..."
(cd assets/account; scrypto test)
(cd assets/sys-faucet; scrypto test)
(cd assets/sys-utils; scrypto test)
(cd examples/hello-world; scrypto test)
(cd examples/no-std; scrypto test)

echo "Running simulator..."
(cd simulator; bash ./tests/resim.sh)
(cd simulator; bash ./tests/scrypto.sh)
(cd simulator; bash ./tests/manifest.sh)

echo "Running benchmark..."
(cd sbor-tests; cargo bench)
(cd radix-engine; cargo bench)

echo "Congrats! All tests passed."
