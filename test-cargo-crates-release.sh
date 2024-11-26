#!/bin/bash
set -e
set -u

echo "Publishing all cargo crates"
CARGO_FILES=(
    "./radix-rust/Cargo.toml"
    "./sbor-derive-common/Cargo.toml"
    "./sbor-derive/Cargo.toml"
    "./sbor/Cargo.toml"
    "./radix-sbor-derive/Cargo.toml"
    "./radix-common/Cargo.toml"
    "./radix-common-derive/Cargo.toml"
    "./radix-blueprint-schema-init/Cargo.toml"
    "./radix-engine-interface/Cargo.toml"
    "./scrypto-derive/Cargo.toml" 
    "./scrypto/Cargo.toml"
    "./radix-substate-store-interface/Cargo.toml"
    "./radix-substate-store-impls/Cargo.toml"
    "./radix-engine-profiling/Cargo.toml"
    "./radix-engine-profiling-derive/Cargo.toml"
    "./radix-native-sdk/Cargo.toml"
    "./radix-transactions/Cargo.toml"
    "./radix-engine/Cargo.toml"
    "./radix-transaction-scenarios/Cargo.toml"
    "./radix-substate-store-queries/Cargo.toml" 
    "./scrypto-bindgen/Cargo.toml"
    "./scrypto-compiler/Cargo.toml"
    "./scrypto-test/Cargo.toml"
    "./radix-clis/Cargo.toml"
)
for toml_file_dir in ${CARGO_FILES[@]}; do
    echo "Publishing crate in directory ${toml_file_dir}"
    # Use --no-verify as the crates haven't been published
    # https://github.com/crate-ci/cargo-release/issues/691#issuecomment-1636475265
    echo "cargo publish --no-verify --dry-run --token "${CRATES_TOKEN}" --manifest-path ${toml_file_dir}"
    cargo publish --dry-run --token "${CRATES_TOKEN}" --manifest-path ${toml_file_dir}
    cargo package --list --manifest-path "${toml_file_dir}"
done
