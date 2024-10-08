#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

rtmc="cargo run --bin rtmc -- $@ "
rtmd="cargo run --bin rtmd -- $@ "

# Decompile and recompile a subintent
$rtmc --output ./tests/out/subintent.bin --kind subintentv2 ./tests/subintent.rtm
$rtmd --output ./tests/out/subintent.rtm ./tests/out/subintent.bin
