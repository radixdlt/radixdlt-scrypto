#!/bin/bash
set -u
set -e
export VERSION=$1

echo "Updating to version ${VERSION}"
cargo install toml-cli
 
RADIX_CARGO_FILES=("./scrypto-derive/Cargo.toml" "./radix-substate-store-queries/Cargo.toml" "./radix-common-derive/Cargo.toml" "./radix-engine-profiling/Cargo.toml" "./radix-substate-store-impls/Cargo.toml" "./radix-clis/Cargo.toml" "./radix-sbor-derive/Cargo.toml" "./sbor-derive/Cargo.toml" "./scrypto/Cargo.toml" "./scrypto-test/Cargo.toml" "./radix-transactions/Cargo.toml" "./radix-native-sdk/Cargo.toml" "./radix-blueprint-schema-init/Cargo.toml" "./scrypto-compiler/Cargo.toml" "./sbor-tests/Cargo.toml" "./radix-engine-interface/Cargo.toml" "./radix-rust/Cargo.toml" "./sbor-derive-common/Cargo.toml" "./radix-engine-profiling-derive/Cargo.toml" "./radix-common/Cargo.toml" "./sbor/Cargo.toml" "./scrypto-derive-tests/Cargo.toml" "./radix-engine/Cargo.toml" "./radix-transaction-scenarios/Cargo.toml" "./radix-engine-monkey-tests/Cargo.toml" "./radix-engine-tests/Cargo.toml" "./radix-substate-store-interface/Cargo.toml")
INTERNAL_PROJECT_LIST=("radix-blueprint-schema-init" "radix-common" "radix-common-derive" "radix-engine" "radix-engine-interface" "radix-engine-profiling" "radix-engine-profiling-derive" "radix-native-sdk" "radix-rust" "radix-sbor-derive" "radix-substate-store-impls" "radix-substate-store-interface" "radix-substate-store-queries" "radix-transaction-scenarios" "radix-transactions" "sbor" "sbor-derive" "sbor-derive-common" "scrypto" "scrypto-compiler" "scrypto-derive" "scrypto-test")
NUMBER_OF_PROJECTS=${#INTERNAL_PROJECT_LIST[@]}


echo "Update the package.version in all radix owned Cargo.toml files"
for toml_file in ${RADIX_CARGO_FILES[@]}; do
    FILENAME=${toml_file}
    echo "Updating ${toml_file} from $(toml get "${FILENAME}" package.version) to \"${VERSION}\""
    toml set "${FILENAME}" package.version "${VERSION}" > "${FILENAME}.new"
    mv "${FILENAME}.new" "${FILENAME}"
done

NUMBER_OF_PACKAGES_IN_LOCKFILE=$(toml get simulator/Cargo.lock package | jq length)

echo "Update the package.version of all radix owned projects in the Cargo.lock file"
for (( i=0; i<$NUMBER_OF_PACKAGES_IN_LOCKFILE; i++ ))
do
    value=$(toml get simulator/Cargo.lock "package[$i].name" -r);
    if [[ "${RADIX_CARGO_FILES[@]}" =~ $value && $value != "toml" ]]; then
        toml set "simulator/Cargo.lock" "package[$i].version" "${VERSION}" > "simulator/Cargo.lock.new"
        mv simulator/Cargo.lock.new simulator/Cargo.lock
    fi;
done

echo "Update workspace dependencies in Cargo.toml"
for (( i=0; i<$NUMBER_OF_PROJECTS; i++ ))
do
    set +e
    value=$(toml get Cargo.toml "workspace.dependencies.${INTERNAL_PROJECT_LIST[$i]}" -r);
    ret=$?
    set -e
    if [ $ret -ne 0 ]; then
        echo "Skipping ${INTERNAL_PROJECT_LIST[$i]}. It is not a dependency"
    else
        echo "File is ${INTERNAL_PROJECT_LIST[$i]} Value is$value"
        toml set Cargo.toml "workspace.dependencies.${INTERNAL_PROJECT_LIST[$i]}.version" "${VERSION}" > Cargo.toml.new
        mv Cargo.toml.new Cargo.toml
    fi
done

echo "Update dependencies of radix-clis/Cargo.toml"
for (( i=0; i<$NUMBER_OF_PROJECTS; i++ ))
do
    set +e
    value=$(toml get radix-clis/Cargo.toml "dependencies.${INTERNAL_PROJECT_LIST[$i]}" -r);
    ret=$?
    set -e
    if [ $ret -ne 0 ]; then
        echo "Skipping ${INTERNAL_PROJECT_LIST[$i]}. It is not a dependency"
    else
        echo "Setting ${INTERNAL_PROJECT_LIST[$i]} version dependency from $value to ${VERSION}"
        toml set radix-clis/Cargo.toml "dependencies.${INTERNAL_PROJECT_LIST[$i]}.version" "${VERSION}" > radix-clis/Cargo.toml.new
        mv radix-clis/Cargo.toml.new radix-clis/Cargo.toml
    fi
done

./update-cargo-locks.sh

echo "Done"


