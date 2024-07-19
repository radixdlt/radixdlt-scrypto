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

echo "Testing crates with no_std..."
test_crates_features \
    "sbor \
    sbor-tests \
    scrypto \
    radix-engine \
    radix-engine-tests" \
    "--no-default-features --features alloc"

echo "Congrats! All extra tests passed."
