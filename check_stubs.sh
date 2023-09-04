#!/bin/bash

set -x
set -e

stubs_path=$PWD/scrypto/src/component/stubs.rs

current_hash=$(md5sum $stubs_path | awk '{print $1}')

./update-bindings.sh
new_hash=$(md5sum $stubs_path | awk '{print $1}')

if [ "$current_hash" == "$new_hash" ]; then
    exit 0
else
    exit 1
fi