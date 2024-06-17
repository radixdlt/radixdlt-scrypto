#!/bin/bash
set -e
set -u

echo "Publishing all cargo crates"
CARGO_FILES=("./scrypto-derive/Cargo.toml" "./radix-substate-store-queries/Cargo.toml" "./radix-common-derive/Cargo.toml" "./radix-engine-profiling/Cargo.toml" "./radix-substate-store-impls/Cargo.toml" "./radix-clis/Cargo.toml" "./radix-sbor-derive/Cargo.toml" "./sbor-derive/Cargo.toml" "./scrypto/Cargo.toml" "./scrypto-test/Cargo.toml" "./radix-transactions/Cargo.toml" "./radix-native-sdk/Cargo.toml" "./radix-blueprint-schema-init/Cargo.toml" "./scrypto-compiler/Cargo.toml" "./sbor-tests/Cargo.toml" "./radix-engine-interface/Cargo.toml" "./radix-rust/Cargo.toml" "./sbor-derive-common/Cargo.toml" "./radix-engine-profiling-derive/Cargo.toml" "./radix-common/Cargo.toml" "./sbor/Cargo.toml" "./scrypto-bindgen/Cargo.toml" "./scrypto-derive-tests/Cargo.toml" "./radix-engine/Cargo.toml" "./radix-transaction-scenarios/Cargo.toml" "./radix-engine-monkey-tests/Cargo.toml" "./radix-engine-tests/Cargo.toml" "./radix-substate-store-interface/Cargo.toml")
for toml_file_dir in ${CARGO_FILES[@]}; do
    echo "Publishing crate in directory ${toml_file_dir}"
    echo "cargo publish --dry-run --token "${CRATES_TOKEN}" --manifest-path ${toml_file_dir}"
    cargo publish --dry-run --token "${CRATES_TOKEN}" --manifest-path ${toml_file_dir}
    cargo package --list --manifest-path "${toml_file_dir}"
done
