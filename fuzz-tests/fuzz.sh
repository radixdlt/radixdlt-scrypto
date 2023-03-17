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

function usage() {
    echo "$0 [FUZZER/COMMAND] [SUBCOMMAND] [COMMAND-ARGS]"
    echo "Available fuzzers"
    echo "    libfuzzer  - 'cargo fuzz' wrapper"
    echo "    afl        - 'cargo afl' wrapper"
    echo "    simple     - simple fuzzer (default)"
    echo "  Subcommands:"
    echo "      init        - Take sample input ('./fuzz_input/<target>') for given test,"
    echo "                    minimize the test corpus and put the result into 'corpus/<target>',"
    echo "                    which is used by 'libfuzzer' as initial input."
    echo "                    Applicable only for 'libfuzzer'."
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
    echo "Available commands"
    echo "    generate-input - generate fuzzing input data"
    echo "  Subcommands:"
    echo "        raw         - Do not process generated data"
    echo "        unique      - Make the input data unique"
    echo "        minimize    - Minimize the input data"
    echo "Examples:"
    echo "  - build AFL fuzz tests"
    echo "    $0 afl build"
    echo "  - run AFL tests for 1h"
    echo "    $0 afl run -V 3600"
    echo "  - run LibFuzzer for 1h"
    echo "    $0 libfuzzer run -max_total_time=3600"
    echo "  - run simple-fuzzer for 1h"
    echo "    $0 simple run --duration 3600"
    echo "  - reproduce some crash discovered by 'libfuzzer'"
    echo "    $0 libfuzzer run ./artifacts/transaction/crash-ec25d9d2a8c3d401d84da65fd2321cda289d"
    echo "  - reproduce some crash discovered by 'libfuzzer' using 'simple-fuzzer'"
    echo "    RUST_BACKTRACE=1 $0 simple run ./artifacts/transaction/crash-ec25d9d2a8c3d401d84da65fd2321cda289d"
}

function error() {
    local msg=$1
    echo "error - $msg"
    usage
    exit 1
}

function fuzzer_libfuzzer() {
    local cmd=${1:-$DFLT_SUBCOMMAND}
    shift
    local run_args=""
    if [ "$cmd" = "run" ] ; then
        run_args="$@"

    elif [ "$cmd" = "init" ] ; then
        # initial setup:
        # - minimize the corpus:
        #    https://llvm.org/docs/LibFuzzer.html#id25
        #
        #   cargo +nightly fuzz $target  --fuzz-dir radix-engine-fuzz \
        #      --no-cfg-fuzzing --target-dir target-libfuzzer $target -- \
        #      -merge=1 corpus/$target <INTERESTING_INPUTS_DIR/FULL_CORPUS_DIR>
        #
        cmd=run
        run_args="-- -merge=1 corpus/${target} fuzz_input/${target} "
    fi
    # Unset cfg=fuzzing by --no-cfg-fuzzing.
    # "secp256k1" uses some stubs instead of true cryptography if "fuzzing" is set.
    # see: https://github.com/rust-bitcoin/rust-secp256k1/#fuzzing
    set -x
    cargo +nightly fuzz $cmd \
        --release \
        --no-default-features --features std,libfuzzer-sys \
        --fuzz-dir . \
        --no-cfg-fuzzing \
        --target-dir target-libfuzzer \
        $target \
        $run_args

}

function fuzzer_afl() {
    local cmd=${1:-$DFLT_SUBCOMMAND}
    local run_args="-T $target "
    local run_cmd_arg=
    shift

    if [ $cmd = "build" ] ; then
        cargo afl build --release \
            --no-default-features --features std,afl \
            --target-dir target-afl
    elif [ $cmd = "run" ] ; then
        mkdir -p afl/${target}/out
        export AFL_AUTORESUME=1
        set -x
        cargo afl fuzz -i fuzz_input/${target} -o afl/${target} $@ -- target-afl/release/${target}
    elif [ $cmd = "machine-init" ] ; then
        uname="$(uname -s)"
        if [ $uname = "Linux" ] ; then
            set -x
            # disable external utilities handling coredumps
            sudo bash -c "echo core > /proc/sys/kernel/core_pattern"
            # disable CPU frequency scaling
            find /sys/devices/system/cpu -name scaling_governor | \
                xargs -I {} sudo bash -c "echo performance > {}"
        elif [ $uname = "Darwin" ] ; then
            echo "If you see an error message like 'shmget() failed' above, try running the following command:"
            echo "  sudo /Users/<username>/.local/share/afl.rs/rustc-<version>/afl.rs-<version>/afl/bin/afl-system-config"
        else
            error "OS '$uname' not supported"
        fi
    fi
}

function fuzzer_simple() {
    local cmd=${1:-$DFLT_SUBCOMMAND}
    local run_args=""
    local run_cmd_arg=
    shift

    set -x
    cargo $cmd --release \
        --no-default-features --features std,simple-fuzzer \
        --bin $target \
        -- $@
}

function generate_input() {
    # available modes: raw, unique, minimize
    local mode=${1:-minimize}
    if [ $target = "transaction" ] ; then
        if [ ! -f target-afl/release/${target} ] ; then
            echo "target binary 'target-afl/release/${target}' not built. Call below command to build it"
            echo "$THIS_SCRIPT afl build"
            exit 1
        fi
        local curr_path=$(pwd)
        local cmin_dir=fuzz_input/${target}_cmin
        local raw_dir=fuzz_input/${target}_raw
        local final_dir=fuzz_input/${target}

        mkdir -p $raw_dir $cmin_dir $final_dir

        pushd ..
        # Collect input data
        cargo nextest run -p radix-engine-tests --features dump_manifest_to_file can_withdraw_from_my_allocated_account
        popd
        if [ $mode = "raw" ] ; then
            mv ../radix-engine-tests/manifest_*.raw ${curr_path}/${final_dir}
            return
        fi

        mv ../radix-engine-tests/manifest_*.raw ${curr_path}/${raw_dir}

        # Make the input corpus unique
        cargo afl cmin -i $raw_dir -o $cmin_dir -- target-afl/release/${target} 2>&1 | tee afl_cmin.log
        if [ $mode = "unique" ] ; then
            mv $cmin_dir/* $final_dir
            return
        fi

        # `cargo afl tmin` expects AFL_MAP_SIZE to be set, we take the value which is used by `cargo afl cmin`
        export AFL_MAP_SIZE=$(grep AFL_MAP_SIZE afl_cmin.log | sed -E 's/^.*AFL_MAP_SIZE=//g')

        # Minimize all corpus files
        pushd $cmin_dir
        # Filter out the files not greater than 100k to reduce minimizing duration
        if which parallel && parallel --version | grep -q 'GNU parallel' ; then
            # parallel is nicer because is serializes output from commands in parallel
            find . -type f -size -100k | parallel -- \
                cargo afl tmin -i "{}" -o "${curr_path}/${final_dir}/{/}" -- ${curr_path}/target-afl/release/${target}
        else
            find . -type f -size -100k | xargs -P 8 -I {} \
                cargo afl tmin -i "{}" -o "${curr_path}/${final_dir}/{}" -- ${curr_path}/target-afl/release/${target}
        fi
        popd
    else
        echo "error: target '$target' not supported"
        exit 1
    fi
}

target=$DFLT_TARGET
# available fuzzers/commands: libfuzzer, afl, simple, generate-input
cmd=${1:-$DFLT_COMMAND}
shift

if [ $cmd = "libfuzzer" ] ; then
    fuzzer_libfuzzer $@
elif [ $cmd = "afl" ] ; then
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
