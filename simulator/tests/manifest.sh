#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset
account=`$resim new-account | awk '/Account component address:/ {print $NF}'`
package=`$resim publish ../examples/hello-world | awk '/Package:/ {print $NF}'`
sed "s/010958e20ce1de938d484e1c585fe95f9470be7befbb02ae829094/$package/g;s/02d4ca16f3a137bc109a16e752aae2bb64a65b318746c6c9199b0c/$account/g" ./tests/m1.rtm > target/m1.rtm


tokens=`$resim run ./target/m1.rtm | awk '/Component:|Resource:/ {print $NF}'`
component=`echo $tokens | cut -d " " -f1`
resource=`echo $tokens | cut -d " " -f2`
sed "s/02d4ca16f3a137bc109a16e752aae2bb64a65b318746c6c9199b0c/$account/g;s/02d28b416875900786bef97fbb43d4997fdc22ae3f34033a35518a/$component/g;s/03cbe9bceec37c8360b27c57f33358bd8b342a956b62429c8c5909/$resource/g" ./tests/m2.rtm > target/m2.rtm
$resim run ./target/m2.rtm

$resim show-ledger