#!/bin/bash

set -e
set -u

ARTIFACT_NAME=fuzz_transaction.tgz
url=$1
run_id=${url##*/}

dir=run_${run_id}

function usage() {
    echo "$0 <run-id>"
    echo "Command speeds up processing fuzzing results."
    echo "The script gets Github run id or URL and checks fuzzing status."
    echo "It also tries to reproduce the crashes if this is the case."
    echo "This is to classify crashes and filter out duplicates."
    echo "  <run-id>  - Github action run id or url"
    echo "  <run-id>  - Github action run id or url"
    exit
}

function validate_run() {
    local view=
    view=$(gh run view $run_id --json status,conclusion,url,workflowName,headBranch,headSha,displayTitle)
    status=$(jq -r '.status' <<< $view)
    conclusion=$(jq -r '.conclusion' <<< $view)
    url=$(jq -r '.url' <<< $view)
    name=$(jq -r '.workflowName' <<< $view)
    branch=$(jq -r '.headBranch' <<< $view)
    sha=$(jq -r '.headSha' <<< $view)

    title="$(jq -r '.displayTitle' <<< $view)"
    if [ $status = "in_progress" ] ; then
        echo "run $run_id still in progress - come back later. Details: $url"
        exit 1
    fi
    if [ $conclusion = "failure" ] ; then
        echo "run $run_id failed - nothing to process. Details: $url"
        exit 1
    fi
    if [ $name != "Fuzzing" ] ; then
        echo "run $run_id is a '$name' not 'Fuzzing' run. Details: $url"
        exit 1
    fi

    echo "Found run:"
    echo "  title : $title"
    echo "  branch: $branch"
    echo "  sha   : $sha"
}

function get_artifacs() {
    echo "Seting up a work dir: $dir"
    mkdir -p $dir

    echo "Downloading $ARTIFACT_NAME"
    gh run download $run_id -n $ARTIFACT_NAME -D $dir

    tar xf $dir/$ARTIFACT_NAME -C $dir
    rm $dir/$ARTIFACT_NAME
}

function get_summary() {
    local d=${1:-}
    local with_files=${2:-yes}
    cat $d/afl/summary | awk '/Summary stats/,/Time without/'
    echo "Fuzzing stats file: $d/afl/summary"

    if [ $with_files = "yes" ] ; then
        echo "  crash/hang files:"
        find $d/afl/*/*/* -name "id*" | xargs -n1 -I {} echo "    "{}
    fi
}

prefixoutput() {
    local prefix="    "
    "$@" > >(sed "s/^/$prefix (stdout): /") 2> >(sed "s/^/$prefix (stderr): /" >&2)
}

function inspect_crashes() {
    echo "Inspecting found crashes"
    pushd $dir
    work_dir=$(pwd)
    #files=$(find $work_dir/afl/*/*/* -name "id*")
    files=$(find afl/*/*/* -name "id*")

    if [ ! -d radixdlt-scrypto ] ; then
        echo "Checking out the repository"
        git clone git@github.com:radixdlt/radixdlt-scrypto.git radixdlt-scrypto
    fi
    git -C radixdlt-scrypto checkout $sha

    pushd radixdlt-scrypto/fuzz-tests
    echo "Building simple fuzzer"
    ./fuzz.sh simple build
    popd
    echo "Checking crash/hangs files"
    for f in $files ; do
        # calling target directly to get rid of unnecessary debugs
        #./fuzz.sh simple run ../../$f >/dev/null || true
        cmd="radixdlt-scrypto/fuzz-tests/target/release/transaction $f"
        echo
        echo "file    : $f"
        echo "command : $cmd"
#        echo "output  :"
#        prefixoutput $cmd || true
        $cmd >output.log 2>&1 || true
        panic=$(grep panic output.log || true)
        echo "panic   : $panic"
        fname=$(echo $panic | sha256sum | awk '{print $1}').panic
        if [ ! -f $fname ] ; then
            echo -e "\npanic   : $panic" > $fname
        fi
        echo "file    : $f" >> $fname
    done

    cat <<EOF > summary.txt
url     : $url
$(get_summary . no)

Crash/hang info
command : radixdlt-scrypto/fuzz-tests/target/release/transaction <file>
$(cat *.panic)
EOF
    rm -f output.log *.panic

    popd

cat <<EOF

## Fuzzing summary
$(cat $dir/summary.txt)

## Processing info
work dir: $dir
summary : $dir/summary.txt
EOF
}

if [ $url = "help" -o $url = "h" ] ; then
    usage
fi
validate_run
get_artifacs
get_summary $dir
inspect_crashes
