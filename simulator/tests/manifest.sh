#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset
account=`$resim new-account | awk '/Account component address:/ {print $NF}'`
package=`$resim publish ../examples/hello-world | awk '/Package:/ {print $NF}'`
sed "s/015c7d0cb306d77130b351005d481923a90448f7843298c7823d2a/$package/g;s/02d4ca16f3a137bc109a16e752aae2bb64a65b318746c6c9199b0c/$account/g" ./tests/m1.rtm > target/m1.rtm


tokens=`$resim run ./target/m1.rtm | awk '/Component:|Resource:/ {print $NF}'`
component=`echo $tokens | cut -d " " -f1`
resource=`echo $tokens | cut -d " " -f2`
sed "s/02d4ca16f3a137bc109a16e752aae2bb64a65b318746c6c9199b0c/$account/g;s/0258a6793957381c8a4951e835093504d8d380881fffa690006c62/$component/g;s/03f90a59c8cc51ff7786d0f9ab6d22f70a885d5ddeea00265b5b6b/$resource/g" ./tests/m2.rtm > target/m2.rtm
$resim run ./target/m2.rtm

$resim show-ledger