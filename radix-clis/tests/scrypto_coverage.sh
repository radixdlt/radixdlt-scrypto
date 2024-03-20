#!/bin/bash

# This script requires rust nightly and wasm32-unknown-unknown target. 
# Additionally, LLVM needs to be installed and its version should match the major version of your rust nightly compiler.

set -x
set -e

cd "$(dirname "$0")/.."

scrypto="cargo run --bin scrypto $@ --"
test_pkg="./target/temp/hello-world"

# Create package
rm -fr $test_pkg
$scrypto new-package hello-world --path $test_pkg --local

# Generate coverage report
$scrypto coverage --path $test_pkg

# Check if coverage report was generated
if [ -f "$test_pkg/coverage/report/index.html" ]; then
    echo "Coverage report generated successfully."
else
    echo "Error: Coverage report not found."
    exit 1
fi

# Clean up
rm -fr $test_pkg