#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

(cargo doc --release --no-deps --document-private-items --package scrypto --package sbor --package radix-engine)

rm -rf ./docs
cp -r ./target/doc ./docs
echo "<meta http-equiv=\"refresh\" content=\"0; url=scrypto\">" > ./docs/index.html
