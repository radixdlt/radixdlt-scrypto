#!/bin/bash

set -x
set -e
set -u

target=$1

get_libfuzzer_corpus_files() {
    local corpus_dir=corpus/${target}
    find $corpus_dir -type f
}

get_afl_corpus_files() {
    local corpus_dir=afl/${target}
    find $corpus_dir -maxdepth 3 -mindepth 3 -path "*/queue/*" -type f
}

run_fuzzer_in_parallel() {
    if which parallel && parallel --version | grep -q 'GNU parallel' ; then
        # parallel is nicer because is serializes output from commands in parallel.
        # "halt now,fail=1" - exit when any job has failed. Kill other running jobs
        parallel --halt now,fail=1 -- \
            ./fuzz.sh simple run $target "{}"
    else
        xargs -P 8 -I {} \
            ./fuzz.sh simple run $target "{}"
    fi
}

export CARGO_INCREMENTAL=0
export RUSTFLAGS='-Cinstrument-coverage'
export LLVM_PROFILE_FILE="fuzz-${target}-%p-%m.profraw"

./fuzz.sh simple build $target

set +e
# Run simple_fuzzer for each corpus file
get_afl_corpus_files | run_fuzzer_in_parallel
get_libfuzzer_corpus_files | run_fuzzer_in_parallel


