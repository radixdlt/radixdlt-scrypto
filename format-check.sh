#!/bin/bash

set -eE

err_report() {
    echo "Something went wrong on line $1"
}

quiet_flag=""

if [ "$1" = "--quiet" ]
  then quiet_flag="--quiet"
fi

trap 'err_report $LINENO' ERR

failed=0
lf=$'\n'

cd "$(dirname "$0")"

# We use the cd trick to avoid issues like this: https://github.com/rust-lang/rustfmt/issues/4432

# This should align with format.sh, build.sh, test.sh, clean.sh, update-cargo-locks-minimally.sh
# ... and at some point, we should replace all these with a shared
#     workspace lister function which returns this list stripped of Cargo.tomls
#     and can be used by all these scripts to save duplication.
packages="Cargo.toml$lf"
packages+="radix-engine-tests/assets/blueprints/Cargo.toml$lf"
packages+="radix-clis/tests/blueprints/Cargo.toml$lf"
packages+="scrypto-test/tests/blueprints/Cargo.toml$lf"
packages+="scrypto-test/assets/blueprints/Cargo.toml$lf"
packages+="scrypto-compiler/tests/assets/scenario_1/Cargo.toml$lf"
packages+="scrypto-compiler/tests/assets/scenario_2/Cargo.toml$lf"
packages+="$(find examples -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \))$lf"

for package in $packages; do
    folder=$(dirname $package)
    (cd $folder; cargo fmt --check $quiet_flag) || { echo "$lf>> Code format check FAILED for $package$lf"; failed=1; }
done

[ $failed -eq 0 ] && echo "Code format check passed!"
exit $failed
