#!/bin/bash

set -x
set -e

CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test --no-fail-fast \
 --package radix-engine-tests \
 --package radix-transactions \
 --package radix-transaction-scenarios \
 --package radix-common \
 --package sbor \
 --package sbor-tests

grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html --excl-br-start "^declare_native_blueprint_state" --excl-start "^declare_native_blueprint_state" --excl-br-stop "^}$" --excl-stop "^}$"
