#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

echo "Testing crates..."
(cd sbor; cargo test)
(cd sbor-derive; cargo test)
(cd sbor-tests; cargo test)
(cd scrypto; cargo test)
(cd scrypto-derive; cargo test)
(cd scrypto-tests; cargo test)
(cd radix-engine-derive; cargo test)
(cd radix-engine-interface; cargo test)
(cd radix-engine; cargo test)
(cd transaction; cargo test)

echo "Testing scrypto packages..."
(cd assets/blueprints/account; scrypto test)
(cd assets/blueprints/faucet; scrypto test)
(cd examples/hello-world; scrypto test)
(cd examples/no-std; scrypto test)

echo "Testing CLIs..."
(cd simulator; bash ./tests/resim.sh)
(cd simulator; bash ./tests/scrypto.sh)
(cd simulator; bash ./tests/manifest.sh)

echo "Testing benchmark..."
(cd sbor-tests; cargo bench)
(cd radix-engine; cargo bench)

echo "Congrats! All tests passed."
