#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

$resim reset
account=`$resim new-account | awk '/Account component address:/ {print $NF}'`
package=`$resim publish ../examples/hello-world | awk '/Package:/ {print $NF}'`
sed "s/017f0ed62fb39fea03583c9051365c1453abb505ec6643a7a070d7/$package/g;s/02d4ca16f3a137bc109a16e752aae2bb64a65b318746c6c9199b0c/$account/g" ./tests/m1.rtm > target/m1.rtm


tokens=`$resim run ./target/m1.rtm | awk '/Component:|Resource:/ {print $NF}'`
component=`echo $tokens | cut -d " " -f1`
resource=`echo $tokens | cut -d " " -f2`
sed "s/02d4ca16f3a137bc109a16e752aae2bb64a65b318746c6c9199b0c/$account/g;s/02bc203e7c80efa58618cdab60dc33f6c4541758803299cbf5e561/$component/g;s/03c1b13b315366a3dab4b3d349fe54d3d65b3d9a89463dbde7f2d6/$resource/g" ./tests/m2.rtm > target/m2.rtm
$resim run ./target/m2.rtm

$resim show-ledger