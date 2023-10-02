#!/bin/bash

#set -x
set -e
set -o pipefail

targets="$1"
if [ $# -ge 1 ] ; then
    targets=$1
    shift
else
    targets=$(./list-fuzz-targets.sh)
fi

get_libfuzzer_corpus_files() {
    local target=$1
    local corpus_dir="corpus/${target}"

    if [ -d $corpus_dir ] ; then
        find $corpus_dir -type f
    fi
}

get_afl_corpus_files() {
    local target=$1
    local corpus_dir="afl/${target}"

    if [ -d $corpus_dir ] ; then
        find $corpus_dir -maxdepth 3 -mindepth 3 -path "*/queue/*" -type f
    fi
}

process_corpus_files() {
    local target=$1
    if which parallel && parallel --version | grep -q 'GNU parallel' ; then
        # parallel is nicer because is serializes output from commands in parallel.
        # "halt never" - continue even if error occurs
        parallel --halt never -- \
            ./fuzz.sh simple run --release $target "{}" || true # true to consume error and not quit
    else
        xargs -P 8 -I {} \
            sh -c './fuzz.sh simple run $target "{}" || true' # true to consume error and not quit
    fi
}

quit=0
for t in $targets ; do
    if ls fuzz-${t}-*.profraw >/dev/null 2>&1 ; then
        echo "WARNING: Some coverage data for target already exist. Please cleanup"
        echo "See: ls fuzz-${t}-*.profraw"
        quit=1
    fi
done
if [ $quit -eq 1 ] ; then
    exit 1
fi

export CARGO_INCREMENTAL=0
export RUSTFLAGS='-Cinstrument-coverage'

for t in $targets ; do
    export LLVM_PROFILE_FILE="fuzz-${t}-%p-%m.profraw"

    list=corpus_files_$t.lst
    rm -f $list
    get_afl_corpus_files $t > $list
    get_libfuzzer_corpus_files $t >> $list

    # If corpus file list not empty then get coverage
    if [ -s $list ] ; then
        echo "Getting code coverage data for target $t"
        ./fuzz.sh simple build --release $t
        cat $list | process_corpus_files $t
    else
        echo "Skipping target $t - no corpus files"
    fi
done

