#!/bin/bash

set -eE

err_report() {
    echo "Something went wrong on line $1"
}

trap 'err_report $LINENO' ERR


failed=0

cd "$(dirname "$0")"

packages=$(cat Cargo.toml | \
    awk '/members/{flag=1;next} /\]/{flag=0} flag' | \
    awk -F '"' '{print $2}')

for package in $packages; do
    # Subdirectories requires --all param (https://github.com/rust-lang/rustfmt/issues/4432)
    if [[ "$package" == */* ]]; then
        all_param="--all"
    else
        all_param=""
    fi
    cargo fmt -p $package --check --quiet $all_param ||
        { echo "Code format check FAILED for $package"; failed=1; }
done

packages="
    assets/blueprints/radiswap/Cargo.toml \
    assets/blueprints/faucet/Cargo.toml \
    examples/hello-world/Cargo.toml \
    examples/no-std/Cargo.toml \
    "
packages+=$(find radix-engine-tests/tests/blueprints -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \))

for package in $packages; do
    cargo fmt --check --quiet --manifest-path $package ||
        { echo "Code format check FAILED for $package"; failed=1; }
done

[ $failed -eq 0 ] && echo "Code format check passed!"
exit $failed
