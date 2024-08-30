#!/bin/bash

set -eE

err_report() {
    echo "Something went wrong on line $1"
}

trap 'err_report $LINENO' ERR


failed=0

cd "$(dirname "$0")"

# NOTE: These should align with `format.sh`

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

packages=""
packages+="radix-clis/tests/blueprints/Cargo.toml"
packages+=$'\n'
packages+="scrypto-compiler/tests/assets/scenario_1/Cargo.toml"
packages+=$'\n'
packages+="scrypto-compiler/tests/assets/scenario_2/Cargo.toml"
packages+=$'\n'
packages+=$(find radix-engine-tests/assets/blueprints -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \))
packages+=$'\n'
packages+=$(find scrypto-test/tests/blueprints -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \))
packages+=$'\n'
packages+=$(find scrypto-test/assets/blueprints -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \))
packages+=$'\n'
packages+=$(find radix-transaction-scenarios/assets/blueprints -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \))
packages+=$'\n'
packages+=$(find examples -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \))

# Uncomment to see all the packages
# echo "$packages";

for package in $packages; do
    cargo fmt --check --quiet --manifest-path $package ||
        { echo "Code format check FAILED for $package"; failed=1; }
done

[ $failed -eq 0 ] && echo "Code format check passed!"
exit $failed
