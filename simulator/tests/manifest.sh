#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset

export account=`$resim new-account | awk '/Account component address:/ {print $NF}'`

$resim run ./tests/setup.rtm

$resim show-ledger

$resim run ./tests/failing.rtm

$resim show-ledger