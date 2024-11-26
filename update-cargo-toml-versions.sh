#!/bin/bash
set -u
set -e
export VERSION=$1

echo "Updating to version ${VERSION}"
cargo install toml-cli

RADIX_CARGO_FILES=("./scrypto-derive/Cargo.toml" "./radix-substate-store-queries/Cargo.toml" "./radix-common-derive/Cargo.toml" "./radix-engine-profiling/Cargo.toml" "./radix-substate-store-impls/Cargo.toml" "./radix-clis/Cargo.toml" "./radix-sbor-derive/Cargo.toml" "./sbor-derive/Cargo.toml" "./scrypto/Cargo.toml" "./scrypto-test/Cargo.toml" "./radix-transactions/Cargo.toml" "./radix-native-sdk/Cargo.toml" "./radix-blueprint-schema-init/Cargo.toml" "./scrypto-bindgen/Cargo.toml" "./scrypto-compiler/Cargo.toml" "./sbor-tests/Cargo.toml" "./radix-engine-interface/Cargo.toml" "./radix-rust/Cargo.toml" "./sbor-derive-common/Cargo.toml" "./radix-engine-profiling-derive/Cargo.toml" "./radix-common/Cargo.toml" "./sbor/Cargo.toml" "./scrypto-derive-tests/Cargo.toml" "./radix-engine/Cargo.toml" "./radix-transaction-scenarios/Cargo.toml" "./radix-engine-monkey-tests/Cargo.toml" "./radix-engine-tests/Cargo.toml" "./radix-substate-store-interface/Cargo.toml" "./radix-engine-toolkit-common/Cargo.toml")
INTERNAL_PROJECT_LIST=("radix-blueprint-schema-init" "radix-common" "radix-common-derive" "radix-engine" "radix-engine-toolkit-common" "radix-engine-interface" "radix-engine-profiling" "radix-engine-profiling-derive" "radix-native-sdk" "radix-rust" "radix-sbor-derive" "radix-substate-store-impls" "radix-substate-store-interface" "radix-substate-store-queries" "radix-transaction-scenarios" "radix-transactions" "sbor" "sbor-derive" "sbor-derive-common" "scrypto" "scrypto-bindgen" "scrypto-compiler" "scrypto-derive" "scrypto-test")

NUMBER_OF_PROJECTS=${#INTERNAL_PROJECT_LIST[@]}

echo "Update workspace dependencies in Cargo.toml"
for (( i=0; i<$NUMBER_OF_PROJECTS; i++ ))
do
    set +e
    value=$(toml get Cargo.toml "workspace.dependencies.${INTERNAL_PROJECT_LIST[$i]}" -r);
    ret=$?
    set -e
    if [ $ret -eq 0 ]; then
        echo "Updating ${INTERNAL_PROJECT_LIST[$i]} from $value to ${VERSION}"
        toml set Cargo.toml "workspace.dependencies.${INTERNAL_PROJECT_LIST[$i]}.version" "${VERSION}" > Cargo.toml.new
        mv Cargo.toml.new Cargo.toml
    fi
done

echo "Update the package.version in all radix owned Cargo.toml files"
for toml_file in ${RADIX_CARGO_FILES[@]}; do
    FILENAME=${toml_file}
    echo "Updating ${toml_file} from $(toml get "${FILENAME}" package.version) to \"${VERSION}\""
    toml set "${FILENAME}" package.version "${VERSION}" > "${FILENAME}.new"
    mv "${FILENAME}.new" "${FILENAME}"
done

./update-cargo-locks-minimally.sh

echo "Done"