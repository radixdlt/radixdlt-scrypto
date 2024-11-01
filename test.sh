#!/bin/bash

set -e

cd "$(dirname "$0")"

# Use nextext if it's available
cargo_test_runner="test"
doc_test_separately=0
if cargo help nextest 2>/dev/null >&2 ; then
    cargo_test_runner="nextest run"

    # Workaround for lack of doctests support for nextest
    # Need to keep it until issue resolved https://github.com/nextest-rs/nextest/issues/16
    doc_test_separately=1
fi

# This should align with format-check.sh, check.sh, build.sh, format.sh, clean.sh, update-cargo-locks-minimally.sh

echo "Running tests..."
(set -x; cd .; cargo $cargo_test_runner)
(set -x; cd radix-engine-tests/assets/blueprints; cargo $cargo_test_runner)
(set -x; cd radix-clis/tests/blueprints; cargo $cargo_test_runner)
(set -x; cd scrypto-test/tests/blueprints; cargo $cargo_test_runner)
(set -x; cd scrypto-test/assets/blueprints; cargo $cargo_test_runner)
(set -x; cd scrypto-compiler/tests/assets/scenario_1; cargo $cargo_test_runner)
(set -x; cd scrypto-compiler/tests/assets/scenario_2; cargo $cargo_test_runner)
(
    find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo $cargo_test_runner --manifest-path {}"
)

if [ $doc_test_separately -eq 1 ] ; then
    echo "Running doctests..."
    (set -x; cd .; cargo test --doc)
    (set -x; cd radix-engine-tests/assets/blueprints; cargo test --doc)
    (set -x; cd radix-clis/tests/blueprints; cargo test --doc)
    (set -x; cd scrypto-test/tests/blueprints; cargo test --doc)
    (set -x; cd scrypto-test/assets/blueprints; cargo test --doc)
    (set -x; cd scrypto-compiler/tests/assets/scenario_1; cargo test --doc)
    (set -x; cd scrypto-compiler/tests/assets/scenario_2; cargo test --doc)
    (
        find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
        | xargs -I '{}' bash -c "set -x; cargo test --doc --manifest-path {}"
    )
fi

echo "Testing CLIs..."
./radix-clis/tests/resim.sh
./radix-clis/tests/scrypto.sh
./radix-clis/tests/manifest.sh

echo "Running benchmarks..."
(set -x; cd .; cargo bench)

echo "Checking stack usage..."
./check_stack_usage.sh

echo "Testing sbor with release profile..."
cargo $test_runner -p sbor --release

echo "Testing crates with no_std..."
cargo $test_runner \
    -p sbor \
    -p sbor-tests \
    -p scrypto \
    -p radix-engine \
    -p radix-engine-tests \
    --no-default-features --features alloc

echo "Congrats! All tests passed."
