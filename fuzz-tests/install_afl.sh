#!/bin/bash

# this is to install a modified 'cargo afl' fuzzer, which does set 'fuzzing' flag
# when compiling fuzzed targets.

if cargo afl help 1>/dev/null 2>&1 ; then
    echo -e "cargo afl already installed. Remove it first with \ncargo uninstall afl"
    exit 1
fi

set -e
tmpdir=$(mktemp -d)
pushd $tmpdir

repo=git@github.com:lrubasze/afl.rs.git
branch=allow_no_cfg_fuzzing

echo "Fetching cargo-afl from $repo"
git clone $repo
pushd afl.rs
git checkout $branch
git submodule update --init AFLplusplus

echo "Installing cargo-afl"
cargo install --path . afl --features no_cfg_fuzzing

popd
popd

rm -rf $tmpdir
