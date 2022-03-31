#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset
$resim new-account
$resim publish ./tests/hello_world.wasm
$resim run ./tests/m1.rtm
$resim run ./tests/m2.rtm
$resim show-ledger