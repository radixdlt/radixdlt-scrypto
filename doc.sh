#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

cargo doc \
    --no-deps \
    --document-private-items \
    --package sbor \
    --package radix-engine \
    --package scrypto;

doc_index="./target/doc/scrypto/index.html";
(xdg-open $doc_index || open $doc_index || start $doc_index);
