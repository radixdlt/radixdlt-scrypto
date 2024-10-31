#!/bin/bash

set -e

cd "$(dirname "$0")"

# This should align with format-check.sh, check.sh, build.sh, test.sh, clean.sh, update-cargo-locks-minimally.sh

(set -x; cd .; cargo fmt)
(set -x; cd radix-engine-tests/assets/blueprints; cargo fmt)
(set -x; cd radix-clis/tests/blueprints; cargo fmt)
(set -x; cd scrypto-test/tests/blueprints; cargo fmt)
(set -x; cd scrypto-test/assets/blueprints; cargo fmt)
(set -x; cd scrypto-compiler/tests/assets/scenario_1; cargo fmt)
(set -x; cd scrypto-compiler/tests/assets/scenario_2; cargo fmt)

(
    find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo fmt --manifest-path {}"
)

echo "All packages have been formatted."
