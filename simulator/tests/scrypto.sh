#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

scrypto="cargo run --bin scrypto $@ --"
test_pkg="./target/temp/hello-world"

# Create package
rm -fr $test_pkg
$scrypto new-package hello-world --path $test_pkg --local

# Build
$scrypto build --path $test_pkg

# Test
$scrypto test --path $test_pkg
$scrypto test --path $test_pkg -- test_hello --nocapture
$scrypto test --path $test_pkg -- --nocapture

# Logging
$scrypto build --path ../examples/everything --log-level ERROR
size1=$(ls -la ../examples/everything/target/wasm32-unknown-unknown/release/everything.wasm | cut -d ' ' -f 5)
$scrypto build --path ../examples/everything --log-level WARN
size2=$(ls -la ../examples/everything/target/wasm32-unknown-unknown/release/everything.wasm | cut -d ' ' -f 5)
$scrypto build --path ../examples/everything --log-level INFO
size3=$(ls -la ../examples/everything/target/wasm32-unknown-unknown/release/everything.wasm | cut -d ' ' -f 5)
$scrypto build --path ../examples/everything --log-level DEBUG
size4=$(ls -la ../examples/everything/target/wasm32-unknown-unknown/release/everything.wasm | cut -d ' ' -f 5)
$scrypto build --path ../examples/everything --log-level TRACE
size5=$(ls -la ../examples/everything/target/wasm32-unknown-unknown/release/everything.wasm | cut -d ' ' -f 5)

if [ $size1 -lt $size2 ] && [ $size2 -lt $size3 ] && [ $size3 -lt $size4 ] && [ $size4 -lt $size5 ] ; then
  echo "Size check is okay"
  exit 0
else
  echo "Invalid sizes: $size1, $size2, $size3, $size4, $size5"
  exit 1
fi

# Clean up
rm -fr $test_pkg