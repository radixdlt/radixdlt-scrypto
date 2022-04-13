#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset
account=`$resim new-account | awk '/Account component address:/ {print $NF}'`
package=`$resim publish ../examples/hello-world | awk '/Package:/ {print $NF}'`
sed "s/0131ee25a20db91f4383b04d4314de5825be90dd5d92c41fa2b83a/$package/g;s/02d4ca16f3a137bc109a16e752aae2bb64a65b318746c6c9199b0c/$account/g" ./tests/m1.rtm > target/m1.rtm


tokens=`$resim run ./target/m1.rtm | awk '/Component:|Resource:/ {print $NF}'`
component=`echo $tokens | cut -d " " -f1`
resource=`echo $tokens | cut -d " " -f2`
sed "s/02d4ca16f3a137bc109a16e752aae2bb64a65b318746c6c9199b0c/$account/g;s/0203c9dd0315d76c21e21f625ae11d4fa1bfcb2e8cc7fc8e7e35fe/$component/g;s/031f6370dd93f330f42875cffab3fb96d2a8179270952af3929ecf/$resource/g" ./tests/m2.rtm > target/m2.rtm
$resim run ./target/m2.rtm

$resim show-ledger