#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/../../"

resim="cargo run --bin resim $@ --"

# Create test accounts and public keys
$resim reset
temp=`$resim new-account | awk '/Account component address:/ {print $NF}'`
account=`echo $temp | cut -d " " -f1`

# Test - publish
package=`$resim publish ../examples/sanity-check | awk '/Package:/ {print $NF}'`
