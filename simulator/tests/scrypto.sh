#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

cargo build --timings --bin scrypto
timings=$(basename $(ls -t target/cargo-timings/cargo-timing-* | head -n1))
mv target/cargo-timings/$timings target/cargo-timings/timings_scrypto_$timings

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

# Clean up
rm -fr $test_pkg
