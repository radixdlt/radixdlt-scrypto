#!/bin/bash

set -x
set -e
set -u

target=$1

get_libfuzzer_corpus_files() {
    local corpus_dir=corpus/${target}
    find $corpus_dir -type f
}

# afl
get_afl_corpus_files() {
    local corpus_dir=afl/${target}
    find $corpus_dir -maxdepth 3 -mindepth 3 -path "*/queue/*" -type f
}

export CARGO_INCREMENTAL=0
export RUSTFLAGS='-Cinstrument-coverage'
export LLVM_PROFILE_FILE="fuzz-${target}-%p-%m.profraw"

./fuzz.sh simple build $target

set +e
# Run simple_fuzzer for each corpus file
if which parallel && parallel --version | grep -q 'GNU parallel' ; then
    # parallel is nicer because is serializes output from commands in parallel.
    # "halt now,fail=1" - exit when any job has failed. Kill other running jobs
    get_afl_corpus_files | parallel --halt now,fail=1 -- \
        ./fuzz.sh simple run $target "{}"
else
    get_afl_corpus_files | xargs -P 8 -I {} \
        ./fuzz.sh simple run $target "{}"
fi

# Collect code coverage data and generate report
grcov --source-dir .. --binary-path ./target/debug/ --output-path ./target/coverage/html \
    --output-types html --branch --ignore-not-existing  \
    --excl-br-start "^declare_native_blueprint_state" --excl-br-stop "^}$" \
    --excl-start "^declare_native_blueprint_state" --excl-stop "^}$" \
    .

