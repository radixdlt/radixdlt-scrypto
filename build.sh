#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

(cd sbor; cargo build; cargo test --no-run)
(cd sbor-derive; cargo build; cargo test --no-run)
(cd sbor-tests; cargo build; cargo test --no-run; cargo bench --no-run)
(cd scrypto; cargo build; cargo test --no-run)
(cd scrypto-derive; cargo build; cargo test --no-run)
(cd scrypto-tests; cargo build; cargo test --no-run)
(cd radix-engine; cargo build; cargo test --no-run; cargo bench --no-run)
(cd radix-engine-stores; cargo build; cargo test --no-run)
(cd transaction; cargo build; cargo test --no-run)
(cd simulator; cargo build; cargo test --no-run)
(cd radix-engine/tests; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' scrypto build --path {})

echo "Building assets and examples..."
(cd assets/account; scrypto build)
(cd assets/sys-faucet; scrypto build)
(cd examples/hello-world; scrypto build)
(cd examples/no-std; scrypto build)
