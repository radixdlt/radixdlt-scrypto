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

echo "Testing radix engine with wasmer..."
test_crates_features \
    "radix-engine \
    radix-engine-tests" \
    "--features wasmer"

echo "Testing crates with no_std..."
test_crates_features \
    "sbor \
    sbor-tests \
    scrypto-schema \
    scrypto-tests \
    radix-engine \
    radix-engine-tests \
    scrypto" \
    "--no-default-features --features alloc"

echo "Congrats! All extra tests passed."
