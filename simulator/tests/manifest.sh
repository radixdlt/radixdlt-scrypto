#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset
account=`$resim new-account | awk '/Account component address:/ {print $NF}'`
package=`$resim publish ../examples/hello-world | awk '/Package:/ {print $NF}'`
sed "s/<<<package>>>/$package/g;s/<<<account>>>/$account/g" ./tests/m1.rtm > target/m1.rtm


tokens=`$resim run ./target/m1.rtm | awk '/Component:|Resource:/ {print $NF}'`
component=`echo $tokens | cut -d " " -f1`
resource=`echo $tokens | cut -d " " -f2`
sed "s/<<<account>>>/$account/g;s/<<<component>>>/$component/g;s/<<<resource>>>/$resource/g" ./tests/m2.rtm > target/m2.rtm
$resim run ./target/m2.rtm

$resim show-ledger