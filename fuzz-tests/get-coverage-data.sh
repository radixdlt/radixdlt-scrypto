#!/bin/bash

#set -x
set -e
set -o pipefail

usage() {
    echo "Usage:"
    echo "  $0 [FUZZ-TARGET] [CORPUS-MODE]"
    echo "  Fuzz target - optional parameter. If empty or 'all' then all available targets will be inspected"
    echo "  Available targets:"
    targets=$(./list-fuzz-targets.sh)
    for t in $targets ; do
      echo "    $t"
    done
    echo "  Corpus modes - optional parameter"
    echo "    cmin - minimize the corpus before getting code coverage (default)"
    echo "    full - use full corpus generated during fuzzing session"
    echo "Examples:"
    echo "  - get coverage data from all available targets using default corpus mode"
    echo "    $0"
    echo "    $0 all"
    echo "  - get coverage data from 'decimal' target using full corpus"
    echo "    $0 decimal full"
}

TIMEOUT=5000
MODE=release

corpus_mode=cmin
targets=all

if [ $# -ge 1 ] ; then
    targets=$1
    shift
fi
if [ "$targets" = "all" ] ; then
    targets=$(./list-fuzz-targets.sh)
fi

if [ $# -ge 1 ] ; then
    if [[ "$1" =~ ^full|cmin$ ]] ; then
        corpus_mode=$1
    else
        echo "invalid corpus mode: $1"
        usage
        exit 1
    fi
    shift
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

minimize_corpus() {
    local target=$1
    local list=$2
    local input_dir=corpus_${target}
    local cmin_dir=corpus_cmin_${target}
    mkdir $input_dir $cmin_dir
    cat $list | xargs -P 8 -I {} cp "{}" $input_dir

    # NOTE! cargo-afl cmin requires "du" command from GNU coreutils (supporting option -b)
    # If not present on macOs follow these steps:
    #   - install coreutils:
    #     brew install coreutils
    #   - add coreutils path to the PATH environmental variable
    #     eg.
    #       echo 'export PATH="/opt/homebrew/opt/coreutils/libexec/gnubin:$PATH"' >> ~/.profile
    cargo afl cmin -t $TIMEOUT -i $input_dir -o $cmin_dir -- target-afl/${MODE}/${target}

    rm -rf $input_dir

    tmp_list=$(mktemp)
    find $cmin_dir -type f > $tmp_list
    cp $tmp_list $list
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
            sh -c "./fuzz.sh simple run --release $target \"{}\" || true" # true to consume error and not quit
    fi
}

quit=0
for t in $targets ; do
    if ls fuzz-${t}-*.profraw >/dev/null 2>&1 ; then
        echo "WARNING: Some coverage data for target \"$t\" already exist. Please cleanup"
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
    list=corpus_files_$t.lst
    rm -f $list
    get_afl_corpus_files $t > $list
    get_libfuzzer_corpus_files $t >> $list

    if [ -s $list ] ; then
        if [ "$corpus_mode" = "cmin" ] ; then
            minimize_corpus $t $list
        fi

        # If corpus file list not empty then get coverage
        if [ -s $list ] ; then
            echo "Getting code coverage data for target $t"
            ./fuzz.sh simple build --release $t

            # above command calls 'cargo build ...' which produces some *.profraw files, which don't want
            rm -f default_*.*profraw

            export LLVM_PROFILE_FILE="fuzz-${t}-%p-%m.profraw"
            cat $list | process_corpus_files $t
            unset LLVM_PROFILE_FILE
        else
            echo "Skipping target $t - no corpus files"
        fi
    else
        echo "Skipping target $t - no corpus files"
    fi
done

