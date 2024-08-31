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
#
# We wish for scrypto new-package to come with a supported lock file, to:
# (1) Improve the reliability of developer first build experience.
# (2) To align scrypto package dependency versions with the engine, to reduce the attack surface of supply chain attacks.
#
# To test that the generated Cargo.lock is good, we run a build with the --locked command below.
# This checks that the templated cargo lock is complete.
# 
# If this line fails, we need to update the Cargo.lock to align with the engine.
# To do that, from the repo root, you should run:
# ```
# cd radix-clis/target/temp/hello-world
# cp ../../../../Cargo.lock .
# cargo run --bin scrypto $@ -- build
# cp Cargo.lock ../../../assets/template/Cargo.lock_template
# ```
# And then manually go in and delete the `[[package]]` definition for `hello-world` - as this gets added automatically
# in the right place by the new-package command. This test should then pass.
$scrypto build --path $test_pkg --locked

# Test
$scrypto test --path $test_pkg --locked
$scrypto test --path $test_pkg --locked -- test_hello --nocapture
$scrypto test --path $test_pkg --locked -- --nocapture

# Logging
$scrypto build --path ../examples/everything --log-level ERROR --locked
size1=$(ls -la ../examples/everything/target/wasm32-unknown-unknown/release/everything.wasm | cut -d ' ' -f 5)
$scrypto build --path ../examples/everything --log-level WARN --locked
size2=$(ls -la ../examples/everything/target/wasm32-unknown-unknown/release/everything.wasm | cut -d ' ' -f 5)
$scrypto build --path ../examples/everything --log-level INFO --locked
size3=$(ls -la ../examples/everything/target/wasm32-unknown-unknown/release/everything.wasm | cut -d ' ' -f 5)
$scrypto build --path ../examples/everything --log-level DEBUG --locked
size4=$(ls -la ../examples/everything/target/wasm32-unknown-unknown/release/everything.wasm | cut -d ' ' -f 5)
$scrypto build --path ../examples/everything --log-level TRACE --locked
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