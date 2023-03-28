#!/bin/bash

# This is to install a 'cargo afl' fuzzer with a flag, which prevents setting 'fuzzing' flag
# when compiling fuzzed targets.
# Installing it forcefully to make sure AFL is built wit the current rustc version
echo "Installing cargo-afl"
cargo install --force afl --features no_cfg_fuzzing

