#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"
source test_utils.sh

echo "Testing scrypto with release profile..."
test_crates_features \
    "sbor" \
    "--release"

echo "Testing raidx engine with wasmer..."
test_crates_features \
    "radix-engine" \
    "--features wasmer"

echo "Testing crates with no_std..."
test_crates_features \
    "sbor \
    sbor-tests \
    scrypto-abi \
    scrypto-tests \
    radix-engine" \
    "--no-default-features --features alloc"

test_crates_features \
    "scrypto" \
    "--no-default-features --features alloc,prelude"

echo "Congrats! All extra tests passed."
