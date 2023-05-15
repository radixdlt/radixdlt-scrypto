#!/bin/bash

set -e
set -u
#set -x

crash_inventory=$1

#run_4895067287/afl/crash_summary.txt | awk -F'\\(| ' '($1 == "panic" && NF >= 16) { print $16}; ($1 == "panic" && NF < 16) { print "timeout"} ; $1 == "count" {print $5}' | paste - -  | awk '{a[$1]+=$2}END {for(x in a)print x"\t"a[x]}' | awk '{x += $2} END {print x}'
crash_types_file=$(dirname $crash_inventory)/crash_types.txt
function summarize_crashes {
    awk '
        {
            a[$1]+=$3;
            total+=$3
        }
        END {
            for(x in a)
                printf "%-40s %10s\n", x, a[x];
            printf "%-40s %10s\n", "Total", total
        }' | \
    sort -r -n -k2

}

function process_crashes {
    local crash_inventories=$@

    echo "Crash types"
    cat $crash_inventories | \
        awk -F'\\(| ' '
            ($1 == "panic" && NF >= 16) { print $16"\n"$NF};
            ($1 == "panic" && NF < 16) { print "Timeout\n"$NF};
            $1 == "count" {print $NF};
            $1 == "list" {print $NF}
            ' | \
        paste - - - - > $crash_types_file
    cat $crash_types_file | summarize_crashes

    printf "\nCrash types per location\n"
    locations=$(cat $crash_types_file | awk '{print $2}' | sort | uniq)
    for l in $locations ; do
        printf "location:    %s\n" $l
        cat $crash_types_file | grep $l | summarize_crashes
        printf "\n"
    done
}

process_crashes $crash_inventory
