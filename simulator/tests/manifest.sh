#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset

export account=`$resim new-account | awk '/Account component address:/ {print $NF}'`
export package=`$resim publish ../examples/hello-world | awk '/Package:/ {print $NF}'`
export xrd=030000000000000000000000000000000000000000000000000004

output=`$resim run ./tests/m1.rtm | awk '/Component:|Resource:/ {print $NF}'`
export component=`echo $output | cut -d " " -f1`
export resource=`echo $output | cut -d " " -f2`

$resim run ./tests/m2.rtm

$resim show-ledger