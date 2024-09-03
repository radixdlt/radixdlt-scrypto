#!/bin/bash

set -e

cd "$(dirname "$0")"

(set -x; cd .; cargo clean)
(set -x; cd radix-engine-tests/assets/blueprints; cargo clean)
(set -x; cd radix-clis/tests/blueprints; cargo clean)
(set -x; cd scrypto-test/tests/blueprints; cargo clean)
(set -x; cd scrypto-test/assets/blueprints; cargo clean)
(set -x; cd scrypto-compiler/tests/assets/scenario_1; cargo clean)
(set -x; cd scrypto-compiler/tests/assets/scenario_2; cargo clean)
(
    find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | awk '{print substr($1, 1, length($1)-length("Cargo.toml"))}' \
    | xargs -I '{}' bash -c "set -x; cd {}; cargo clean"
)
