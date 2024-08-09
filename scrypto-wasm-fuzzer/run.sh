#!/bin/bash

cargo build --release
mkdir -p input
mkdir -p output
honggfuzz/honggfuzz -P -i input -o output --keep_output -t 600000 -n 1 -v -- ../target/release/wasm_fuzzer