#!/bin/bash

set -x
set -e

check_val() {
b=$(echo $component | sed 's/^.*INFO ] '$1': \(\S*\).*/\1/')
if [[ $b -ne $2 ]]; then
		echo "$1 should be $2"
		exit 1
fi
}

cd "$(dirname "$0")/../.."

resim="cargo run --bin resim $@ --"

# Create test accounts and public keys
$resim reset
temp=`$resim new-account | awk '/Account component address:/ {print $NF}'`
account=`echo $temp | cut -d " " -f1`

# Test - publish, call-function
package=`$resim publish ../examples/math-ops-check | awk '/Package:/ {print $NF}'`
component=`$resim call-function $package Hello a 5`
check_val b 5
check_val c 1
check_val d 5
check_val f 105
check_val g 95
check_val h 500
check_val i 20
check_val j 0
check_val k 25


