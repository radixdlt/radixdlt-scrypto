#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

scrypto="cargo run --bin scrypto $@ --"
test_pkg="./target/temp/hello-world"

# Create package
rm -fr $test_pkg
$scrypto new-package hello-world --path $test_pkg

# Build
$scrypto build --path $test_pkg

# Test
$scrypto test --path $test_pkg
