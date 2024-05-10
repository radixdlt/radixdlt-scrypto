#!/bin/bash
set -e
set -u
export VERSION=$1

echo "Updating to version ${VERSION}"
cargo install toml-cli

FILES=("monkey-tests/Cargo.toml" "native-sdk/Cargo.toml" "radix-engine-common/Cargo.toml" "radix-engine-derive/Cargo.toml" "radix-engine-interface/Cargo.toml" "radix-engine-macros/Cargo.toml" "radix-engine-profiling/Cargo.toml" "radix-engine-queries/Cargo.toml" "radix-engine-store-interface/Cargo.toml" "radix-engine-stores/Cargo.toml" "radix-engine-tests/Cargo.toml" "radix-engine/Cargo.toml" "sbor-derive-common/Cargo.toml" "sbor-derive/Cargo.toml" "sbor-tests/Cargo.toml" "sbor/Cargo.toml" "scrypto-derive-tests/Cargo.toml" "scrypto-derive/Cargo.toml" "scrypto-schema/Cargo.toml" "scrypto-test/Cargo.toml" "scrypto-unit/Cargo.toml" "scrypto/Cargo.toml" "simulator/Cargo.toml" "transaction-scenarios/Cargo.toml" "transaction/Cargo.toml" "utils/Cargo.toml" "radix-engine-profiling/resources-tracker-macro/Cargo.toml" "radix-engine/wasm-benchmarks-lib/Cargo.toml")
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

echo "Done"