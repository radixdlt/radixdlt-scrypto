#!/bin/bash

# Installing cargo-afl 0.14.1 to align with the afl crate used by fuzzer.
# TODO:
# Investigate if lack of 'no_cfg_fuzzing' still affects non-determinism.
# Some context:
#  In 0.12.16 we added 'no_cfg_fuzzing' flag to cargo-afl (details: https://github.com/rust-fuzz/afl.rs/pull/306)
#  to prevents setting 'fuzzing' flag when compiling fuzzed targets (to fight non-determinism)
#
# Installing it forcefully to make sure AFL is built wit the current rustc version

echo "Installing cargo-afl"
cargo install --force cargo-afl  --version 0.14.1
