#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset
account=`$resim new-account | tee /dev/tty | awk '/Account component address:/ {print $NF}'`
package=`$resim publish ./tests/hello_world.wasm | tee /dev/tty | awk '/Package:/ {print $NF}'`
sed "s/01c83c78f073a838898a5419e4ba9a8921ab5e928303189a9f2eaf/$package/g;s/02909c0e4d5160b44dd72b7ad1366087c74ff2d52a8f3de9996512/$account/g" ./tests/m1.rtm > target/m1.rtm


tokens=`$resim run ./target/m1.rtm | tee /dev/tty | awk '/Component:|Resource:/ {print $NF}'`
component=`echo $tokens | cut -d " " -f1`
resource=`echo $tokens | cut -d " " -f2`
sed "s/02909c0e4d5160b44dd72b7ad1366087c74ff2d52a8f3de9996512/$account/g;s/02c6f5b89ea519ef26b229e0163446230df2049959e99f395f1396/$component/g;s/0390a1644d70bda3a64bf4260fd238d32973ce735dbef74a25ce8f/$resource/g" ./tests/m2.rtm > target/m2.rtm
$resim run ./target/m2.rtm

$resim show-ledger