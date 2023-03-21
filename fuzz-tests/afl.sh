#!/bin/bash

set -x
set -e
set -u

DFLT_CPUS=1
DFLT_INTERVAL=60
# At the moment this is the only supported test
DFLT_TARGET=transaction
DFLT_AFL_TIMEOUT=1000

function usage() {
    echo "$0 [COMMAND] [COMMAND-ARGS]"
    echo "Commands:"
    echo "    run <duration> <instances> [timeout]"
    echo "            Run given number of AFL instances (default: $DFLT_CPUS) in screen sessions"
    echo "            for a given number of seconds."
    echo "            For 'instances' one can specify"
    echo "              all      - to run as many instances as CPU cores available"
    echo "              <number> - to run <number> of instances"
    echo "            'timeout' is an AFL timeout in ms"
    echo "    watch - Monitor AFL instances until they are finished."
    echo "            One can specify interval (default: $DFLT_INTERVAL) to output the status"
}

function get_cpus() {
    local uname="$(uname -s)"
    if [ $uname = "Linux" ] ; then
        cat /proc/cpuinfo  | grep processor | wc -l
    elif [ $uname = "Darwin" ] ; then
        sysctl -n hw.ncpu
    else
        echo "OS $uname not supported"
        exit 1
    fi
}

target=$DFLT_TARGET
cmd=${1:-watch}
shift

if [ $cmd = "run" ] ; then
    duration=${1}
    cpus=${2:-1}
    timeout=${3:-$DFLT_AFL_TIMEOUT}
    if [ $cpus = "all" ] ; then
        cpus=$(get_cpus)
        echo "CPU cores available: $cpus"
    fi
    echo "Running $cpus AFL instances"
    mkdir -p afl

    for (( i=0; i<$cpus; i++ )) ; do
        if [ $i -eq 0 ] ; then
            name=main_$i
            # main fuzzer
            fuzzer="-M $name"
        else
            name=secondary_$i
            # secondary fuzzer
            fuzzer="-S $name"
        fi
        # TODO: use different fuzzing variants per instance
        screen -dmS afl_$name \
            bash -c "{ ./fuzz.sh afl run -V $duration $fuzzer -T $target -t $timeout >afl/$name.log 2>afl/$name.err ; echo \$? > afl/$name.status; }"
    done
elif [ $cmd = "watch" ] ; then
    interval=${1:-$DFLT_INTERVAL}
    while ! screen -ls afl | grep "No Sockets found" ; do
        sleep $interval
        # afl folder structure created with some delay after fuzz startup
        if [ -d afl/$target ] ; then
            cargo afl whatsup -d afl/$target
        fi
    done
    echo "AFL instances status (0 means 'ok'):"
    find afl -name "*.status" | xargs grep -H -v "*"
else
    echo "Command '$cmd' not supported"
    exit 1
fi


