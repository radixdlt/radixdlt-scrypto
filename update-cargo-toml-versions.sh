#!/bin/bash
set -e
set -u
export VERSION=$1

echo "Updating to version ${VERSION}"
cargo install toml-cli
 

FILES=("./scrypto-derive/Cargo.toml" "./radix-substate-store-queries/Cargo.toml" "./radix-common-derive/Cargo.toml" "./radix-engine-profiling/Cargo.toml" "./radix-substate-store-impls/Cargo.toml" "./radix-clis/Cargo.toml" "./radix-sbor-derive/Cargo.toml" "./sbor-derive/Cargo.toml" "./scrypto/Cargo.toml" "./scrypto-test/Cargo.toml" "./radix-transactions/Cargo.toml" "./radix-native-sdk/Cargo.toml" "./radix-blueprint-schema-init/Cargo.toml" "./scrypto-compiler/Cargo.toml" "./sbor-tests/Cargo.toml" "./radix-engine-interface/Cargo.toml" "./radix-rust/Cargo.toml" "./sbor-derive-common/Cargo.toml" "./radix-engine-profiling-derive/Cargo.toml" "./radix-common/Cargo.toml" "./sbor/Cargo.toml" "./scrypto-derive-tests/Cargo.toml" "./radix-engine/Cargo.toml" "./radix-transaction-scenarios/Cargo.toml" "./radix-engine-monkey-tests/Cargo.toml" "./radix-engine-tests/Cargo.toml" "./radix-substate-store-interface/Cargo.toml")
for toml_file in ${FILES[@]}; do
    FILENAME=${toml_file}
    mv "${FILENAME}" "${FILENAME}.old"
    echo "Updating ${toml_file} from $(toml get "${FILENAME}.old" package.version) to \"${VERSION}\""
    toml set "${FILENAME}.old" package.version "${VERSION}" > "${FILENAME}"
rm "${FILENAME}.old"
done

maxi=$(toml get simulator/Cargo.lock package | jq length)
for (( i=0; i<$maxi; i++ ))
do
value=$(toml get simulator/Cargo.lock "package[$i].name" -r);
if [[ "${FILES[@]}" =~ $value && $value != "toml" ]]; then
    toml set "simulator/Cargo.lock" "package[$i].version" "${VERSION}" > "simulator/Cargo.lock.new"
    mv simulator/Cargo.lock.new simulator/Cargo.lock
fi;
done
rm -f simulator/Cargo.lock.new

WORKSPACE_DEPENDENCIES=("radix-blueprint-schema-init" "radix-common" "radix-common-derive" "radix-engine" "radix-engine-interface" "radix-engine-profiling" "radix-engine-profiling-derive" "radix-native-sdk" "radix-rust" "radix-sbor-derive" "radix-substate-store-impls" "radix-substate-store-interface" "radix-substate-store-queries" "radix-transaction-scenarios" "radix-transactions" "sbor" "sbor-derive" "sbor-derive-common" "scrypto" "scrypto-compiler" "scrypto-derive" "scrypto-test")
echo "Update workspace dependencies"
maxi=${#WORKSPACE_DEPENDENCIES[@]}
for (( i=1; i<$maxi; i++ ))
do
    value=$(toml get Cargo.toml "workspace.dependencies.${WORKSPACE_DEPENDENCIES[$i]}" -r);
    echo "File is ${WORKSPACE_DEPENDENCIES[$i]} Value is$value"
    toml set Cargo.toml "workspace.dependencies.${WORKSPACE_DEPENDENCIES[$i]}.version" "${VERSION}" > Cargo.toml.new
    mv Cargo.toml.new Cargo.toml
done
rm -f Cargo.toml.new

WORKSPACE_DEPENDENCIES=("radix-blueprint-schema-init" "radix-common" "radix-common-derive" "radix-engine" "radix-engine-interface" "radix-engine-profiling" "radix-engine-profiling-derive" "radix-native-sdk" "radix-rust" "radix-sbor-derive" "radix-substate-store-impls" "radix-substate-store-interface" "radix-substate-store-queries" "radix-transaction-scenarios" "radix-transactions" "sbor" "sbor-derive" "sbor-derive-common" "scrypto" "scrypto-compiler" "scrypto-derive" "scrypto-test")
echo "Update dependencies of radix-clis"
maxi=${#WORKSPACE_DEPENDENCIES[@]}
for (( i=1; i<$maxi; i++ ))
do
    value=$(toml get radix-clis/Cargo.toml "dependencies.${WORKSPACE_DEPENDENCIES[$i]}" -r);
    echo "File is ${WORKSPACE_DEPENDENCIES[$i]} Value is$value"
    toml set radix-clis/Cargo.toml "dependencies.${WORKSPACE_DEPENDENCIES[$i]}.version" "${VERSION}" > radix-clis/Cargo.toml.new
    mv radix-clis/Cargo.toml.new radix-clis/Cargo.toml
done
rm -f radix-clis/Cargo.toml.new

./update-cargo-locks.sh

echo "Done"


