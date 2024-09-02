#!/bin/bash

set -e

# This script causes minimal updates to sync up all the cargo locks

# This script can be run as a quick-fix for an error such as:
# > the lock file [PATH]/Cargo.lock needs to be updated but --locked was passed to prevent this
# > If you want to try to generate the lock file without accessing the network, remove the --locked flag and use --offline instead.

# This should align with format.sh, check.sh, build.sh, test.sh

(set -x; cd .; cargo update --workspace)

(set -x; cd radix-engine-tests/assets/blueprints; cargo update --workspace)
(set -x; cd radix-clis/tests/blueprints; cargo update --workspace)
(set -x; cd scrypto-test/tests/blueprints; cargo update --workspace)
(set -x; cd scrypto-test/assets/blueprints; cargo update --workspace)
(set -x; cd scrypto-compiler/tests/assets/scenario_1; cargo update --workspace)
(set -x; cd scrypto-compiler/tests/assets/scenario_2; cargo update --workspace)

(
    find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo update --workspace --manifest-path {}"
)
