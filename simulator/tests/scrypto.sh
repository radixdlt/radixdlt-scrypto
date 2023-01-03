#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

cargo build --timings --bin scrypto
timings=$(basename $(ls -t target/cargo-timings/cargo-timing-* | head -n1))
mv target/cargo-timings/$timings target/cargo-timings/timings_scrypto_$timings

scrypto="cargo run --bin scrypto $@ --"
test_pkg="./target/temp/hello-world"

ls -la
ls -la ./target || true
ls -la ./target/temp || true
ls -la ./target/temp/hello-world || true
ls -la ./target/temp/hello-world/target || true

if [ -d $test_pkg/target ] ; then
    mv $test_pkg/target scrypto_target
fi
# Create package
rm -fr $test_pkg
$scrypto new-package hello-world --path $test_pkg --local

if [ -d scrypto_target ] ; then
    mv scrypto_target $test_pkg/target
fi

# Build
$scrypto build --path $test_pkg

# Test
$scrypto test --path $test_pkg
$scrypto test --path $test_pkg -- test_hello --nocapture
$scrypto test --path $test_pkg -- --nocapture

# Clean up
#rm -fr $test_pkg
