#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

cd radix-engine;
cargo doc --no-deps --package scrypto --package sbor --package radix-engine;
doc_index="./target/doc/scrypto/index.html";
(xdg-open $doc_index || open $doc_index || start $doc_index);
