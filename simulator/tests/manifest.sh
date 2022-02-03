#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset
$resim new-account
$resim publish ../examples/core/gumball-machine
$resim run ./tests/manifest.rtm
$resim show-ledger