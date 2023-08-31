#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

rm -rf ./target/doc
(cargo doc --release --no-deps --document-private-items \
    --package sbor  \
    --package scrypto \
    --package scrypto-test \
    --package scrypto-unit \
    --package radix-engine-common \
    --package radix-engine-interface \
    --package radix-engine-store-interface \
    --package radix-engine-stores \
    --package radix-engine-queries \
    --package radix-engine \
)

rm -rf ./docs
cp -r ./target/doc ./docs
echo "<meta http-equiv=\"refresh\" content=\"0; url=scrypto\">" > ./docs/index.html
