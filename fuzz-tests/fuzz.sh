#!/bin/bash

#set -x
set -e
set -o pipefail
set -u

THIS_SCRIPT=$0

# defaults
DFLT_COMMAND=simple
DFLT_SUBCOMMAND=run
DFLT_RUN_CMD_ARG=inf
DFLT_TARGET=transaction
DFLT_TIMEOUT=1000

function usage() {
    echo "$0 [FUZZER/COMMAND] [SUBCOMMAND] [OPTIONS] [FUZZ-TARGET] [COMMAND-ARGS]"
    echo "Available targets:"
    targets=$(./list-fuzz-targets.sh)
    for t in $targets ; do
      echo "    $t"
    done
    echo "Available fuzzers"
    echo "    afl        - 'cargo afl' wrapper"
    echo "    simple     - simple fuzzer (default)"
    echo "  Subcommands:"
    echo "      build       - Build fuzz test for given fuzzer."
    echo "                    Binaries are built in 'release' format."
    echo "      run         - Run fuzz test for given fuzzer (default command)"
    echo "                    It takes arguments that might be supplied to the specified fuzzers."
    echo "                    For more information try:"
    echo "                      $0 [FUZZER] run -h"
    echo "     machine-init - Initialize the OS accordingly"
    echo "                    In case of Linux:"
    echo "                    - disable external utils handling coredumps"
    echo "                    - disable CPU frequency scaling"
    echo "                    Applcable only for 'afl'"
    echo "  Available options:"
    echo "     --release     - Build/run fuzzer in release mode."
    echo "                    Applicable only for 'simple' fuzzer"
    echo "Available commands"
    echo "    generate-input - generate fuzzing input data for given target"
    echo "  Subcommands:"
    echo "        empty       - Empty input"
    echo "        raw         - Do not process generated data"
    echo "        unique      - Make the input data unique"
    echo "        minimize    - Minimize the input data"
    echo "  Args:"
    echo "        timeout     - timeout in ms"
    echo "Examples:"
    echo "  - build AFL fuzz tests"
    echo "    $0 afl build transaction"
    echo "  - run AFL tests for 1h"
    echo "    $0 afl run transaction -V 3600"
    echo "  - run simple-fuzzer for 1h"
    echo "    $0 simple run transaction --duration 3600"
    echo "  - run simple-fuzzer in 'release' mode for 1h"
    echo "    $0 simple run --release transaction --duration 3600"
    echo "  - reproduce some crash discovered by 'afl' using 'simple-fuzzer'"
    echo "    RUST_BACKTRACE=1 $0 simple run transaction afl/transaction/0_fast/crashes/id:000168,sig:06,src:001128+000312,time:260091,execs:21509,op:splice,rep:8"
    echo "  - reproduce some crash discovered by 'afl' using 'afl'"
    echo "    cat afl/transaction/0_fast/queue/id:000001,time:0,execs:0,orig:system_001.raw | ./target-afl/release/transaction"

    exit 1
}

function error() {
    local msg=$1
    echo "error - $msg"
    usage
}

function check_target_available() {
    local target=$1
    targets=$(./list-fuzz-targets.sh)
    for t in $targets ; do
        if [ "$t" = "$target" ] ; then
            return 0
        fi
    done
    return 1
}

function fuzzer_afl() {
    local cmd=$DFLT_SUBCOMMAND
    if [ $# -ge 1 ] ; then
        cmd=$1
        shift
    fi
    local target=$DFLT_TARGET
    if [ $# -ge 1 ] ; then
        target=$1
        shift
    fi

    if [ $cmd = "build" ] ; then
        set -x
        cargo afl build --release \
            --bin $target \
            --no-default-features --features std,afl \
            --target-dir target-afl
    elif [ $cmd = "run" ] ; then
        mkdir -p afl/${target}/out
        export AFL_AUTORESUME=1
        set -x
        cargo afl fuzz -i fuzz_input/${target} -o afl/${target} $@ -- target-afl/release/${target}
    elif [ $cmd = "machine-init" ] ; then
        cargo afl system-config
    fi
}

function fuzzer_simple() {
    local cmd=$DFLT_SUBCOMMAND
    if [ $# -ge 1 ] ; then
        cmd=$1
        shift
    fi
    local mode=""
    if [ $# -ge 1 ] ; then
        if [ "$1" = "--release" ] ; then
            mode="--release"
            shift
        fi
    fi
    local target=$DFLT_TARGET
    if [ $# -ge 1 ] ; then
        target=$1
        shift
    fi

    set -x
    cargo $cmd $mode \
        --no-default-features --features std,simple-fuzzer \
        --bin $target \
        -- $@
}

function generate_input() {
    local target=$DFLT_TARGET
    if [ $# -ge 1 ] ; then
        target=$1
        shift
    fi
    # available modes: raw, unique, minimize
    local mode=${1:-minimize}
    local timeout=${2:-$DFLT_TIMEOUT}
    local curr_path=$(pwd)
    local cmin_dir=fuzz_input/${target}_cmin
    local raw_dir=fuzz_input/${target}_raw
    local final_dir=fuzz_input/${target}

    if [ $mode = "empty" ] ; then
        echo "creating empty input $final_dir"
        mkdir -p $final_dir
        # Cannot be empty, let's use newline (0xA).
        echo "" > ${final_dir}/empty
        return
    fi

    if check_target_available $target ; then
        # in 'raw' mode we don't need afl binary
        if [ ! -f target-afl/release/${target} -a $mode != "raw" ]  ; then
            echo "target binary 'target-afl/release/${target}' not built. Call below command to build it"
            echo "$THIS_SCRIPT afl build $target"
            exit 1
        fi

        mkdir -p $raw_dir $cmin_dir $final_dir
        if [ "$(ls -A ${curr_path}/${raw_dir})" ] ; then
            echo "raw dir is not empty, skipping generation"
            if [ $mode = "raw" ] ; then
                find ${curr_path}/${raw_dir} -type f -name "*" | xargs  -I {} mv {} ${curr_path}/${final_dir}
                return
            fi
        fi


        if [ $target != "wasm_instrument" ] ; then
            # Collect input data
            cargo nextest run --no-default-features --features std,simple-fuzzer test_${target}_generate_fuzz_input_data  --release

            if [ $mode = "raw" ] ; then
                #mv ../radix-engine-tests/manifest_*.raw ${curr_path}/${final_dir}
                mv ${target}_*.raw ${curr_path}/${final_dir}
                return
            fi

            #mv ../radix-engine-tests/manifest_*.raw ${curr_path}/${raw_dir}
            mv ${target}_*.raw ${curr_path}/${raw_dir}

        else
            # TODO generate more wasm inputs. and maybe smaller
            if [ $mode = "raw" ] ; then
                find .. -name   "*.wasm" | while read f ; do cp $f $final_dir ; done
                return
            else
                find .. -name   "*.wasm" | while read f ; do cp $f $raw_dir ; done
            fi
        fi

        # do not minimize big files, move them directly to input
        find ${curr_path}/${raw_dir} -type f -size +100k | xargs -I {} mv "{}" ${curr_path}/${final_dir}

        # Make the input corpus unique
        cargo afl cmin -t $timeout -i $raw_dir -o $cmin_dir -- target-afl/release/${target} 2>&1 | tee afl_cmin.log
        if [ $mode = "unique" ] ; then
            mv $cmin_dir/* $final_dir
            return
        fi

        # if `cargo afl cmin` sets the AFL_MAP_SIZE, then set it also for `cargo afl tmin`
        AFL_MAP_SIZE=$(grep AFL_MAP_SIZE afl_cmin.log | sed -E 's/^.*AFL_MAP_SIZE=//g' || true)
        if [ "$AFL_MAP_SIZE" != "" ] ; then
            export AFL_MAP_SIZE
        fi

        # Minimize all corpus files
        pushd $cmin_dir
        # Filter out the files not greater than 100k to reduce minimizing duration
        if which parallel && parallel --version | grep -q 'GNU parallel' ; then
            # parallel is nicer because is serializes output from commands in parallel.
            # "halt now,fail=1" - exit when any job has failed. Kill other running jobs
            find . -type f -size -100k | parallel --halt now,fail=1 -- \
                cargo afl tmin -t $timeout -i "{}" -o "${curr_path}/${final_dir}/{/}" -- ${curr_path}/target-afl/release/${target}
        else
            find . -type f -size -100k | xargs -P 8 -I {} \
                cargo afl tmin -t $timeout -i "{}" -o "${curr_path}/${final_dir}/{}" -- ${curr_path}/target-afl/release/${target}
        fi
        popd
    else
        echo "error: target '$target' not supported"
        exit 1
    fi
}

if [ $# -ge 1 ] ; then
    # available fuzzers/commands: afl, simple, generate-input
    cmd=$1
    shift
else
    cmd=$DFLT_COMMAND
fi

if [ $# -eq 0 ] ; then
    usage
fi

if [ $cmd = "afl" ] ; then
    fuzzer_afl $@
elif [ $cmd = "simple" ] ; then
    fuzzer_simple $@
elif [ $cmd = "generate-input" ] ; then
    generate_input $@
else
    if [ $cmd != "help" -a $cmd != "h" ] ; then
        error "invalid command '$cmd' specified"
    fi
    usage
fi
