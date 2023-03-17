#!/bin/bash

# this is to install a 'cargo afl' fuzzer with a flag, which prevents setting 'fuzzing' flag
# when compiling fuzzed targets.

if cargo afl help 1>/dev/null 2>&1 ; then
    echo -e "cargo afl already installed. Remove it first with \ncargo uninstall afl"
    exit 1
fi

echo "Installing cargo-afl"
cargo install afl --features no_cfg_fuzzing

