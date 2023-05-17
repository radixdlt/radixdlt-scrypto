#!/bin/bash

# This is to install a 'cargo afl' fuzzer with a flag, which prevents setting 'fuzzing' flag
# when compiling fuzzed targets.
# Installing it forcefully to make sure AFL is built wit the current rustc version

# NOTE: forcing version 0.12.17, since current version: 0.13.0 suffers from below error:
#   [-] PROGRAM ABORT : Timeout while initializing fork server (setting AFL_FORKSRV_INIT_TMOUT may help)
#            Location : afl_fsrv_start(), src/afl-forkserver.c:1184
echo "Installing cargo-afl"
cargo install --force afl  --version 0.12.17 --features no_cfg_fuzzing

