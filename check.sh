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
    cargo fmt -p $package --check --quiet ||
        { echo "Code format check FAILED for $package"; failed=1; }
done

packages="
    assets/blueprints/account \
    assets/blueprints/faucet \
    "
packages+=$(find examples -maxdepth 1 -type d \( ! -name . \))
packages+=$(find radix-engine/tests/blueprints -maxdepth 1 -type d \( ! -name . \))

for package in $packages; do
    scrypto fmt --check --quiet --path $package ||
        { echo "Code format check FAILED for $package"; failed=1; }
done

[ $failed -eq 0 ] && echo "Code format check passed!"
exit $failed
