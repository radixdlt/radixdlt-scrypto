#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

(cd sbor; cargo build)
(cd sbor-derive; cargo build)
(cd sbor-tests; cargo build)
(cd scrypto; cargo build)
(cd scrypto-derive; cargo build)
(cd scrypto-tests; cargo build)
(cd radix-engine; cargo build)
(cd transaction; cargo build)

echo "Building assets and examples..."
(cd assets/account; cargo build --target wasm32-unknown-unknown --release)
(cd assets/system; cargo build --target wasm32-unknown-unknown --release)
(cd examples/hello-world; cargo build --target wasm32-unknown-unknown --release)
(cd examples/no-std; cargo build --target wasm32-unknown-unknown --release)
