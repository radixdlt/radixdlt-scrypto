#!/bin/bash

#set -x
set -e

cd "$(dirname "$0")"
source test_utils.sh

setup_test_runner

echo "Testing sbor with release profile..."
test_crates_features \
    "sbor" \
    "--release"

echo "Testing sbor with indexmap..."
test_crates_features \
    "sbor" \
    "--features indexmap"

echo "Testing radix engine with wasmer..."
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
