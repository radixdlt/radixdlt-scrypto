#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

(cd sbor; cargo build; cargo test --no-run)
(cd sbor-derive; cargo build; cargo test --no-run)
(cd sbor-tests; cargo build; cargo test --no-run)
(cd scrypto; cargo build; cargo test --no-run)
(cd scrypto-derive; cargo build; cargo test --no-run)
(cd scrypto-tests; cargo build; cargo test --no-run)
(cd radix-engine; cargo build; cargo test --no-run)
(cd transaction; cargo build; cargo test --no-run)
(cd simulator; cargo build; cargo test --no-run)

echo "Building assets and examples..."
(cd assets/account; cargo build --target wasm32-unknown-unknown --release)
(cd assets/system; cargo build --target wasm32-unknown-unknown --release)
(cd examples/hello-world; cargo build --target wasm32-unknown-unknown --release)
(cd examples/no-std; cargo build --target wasm32-unknown-unknown --release)
