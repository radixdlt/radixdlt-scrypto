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
    # Parse crash inventory file to get following information:
    # - event info (crash kind or timeout)
    # - location in sources (in case of crash)
    # - number of such events
    # - name of the file including the list of the crash files causing this event

    # Exemplary entries in crash inventory file depending on event type:
    # - Crash
    #   column  #1                                                                                #16                                                                      Last (NF in awk)
    #           panic   : thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: MismatchingMapKeyValueKind { key_value_kind: 2, actual_value_kind: 1 }', /home/radixdlt/_work/radixdlt-scrypto/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:62:45
    #           count   : 2
    #           list    : 00b8778915d6dd5b50f2da62eb9d8c89054d5b0ad84f099f0a645ee85e7aa503.panic
    # - Timeout
    #       panic   :
    #       count   : 2
    #       list    : 01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b.panic

    # Generate intermediate file for easier further processing
    cat $crash_inventories | \
        awk -F'\\(| ' '
            ($1 == "panic" && NF >= 16) { print $16"\n"$NF};
            ($1 == "panic" && NF < 16) { print "Timeout\nUnknown"};
            $1 == "count" {print $NF};
            $1 == "list" {print $NF}
            ' | \
        paste - - - - > $crash_types_file

    # Summarize collected information
    echo "# Crash types"
    cat $crash_types_file | summarize_crashes

    printf "\n# Crash types per location\n"
    locations=$(cat $crash_types_file | awk '{print $2}' | sort | uniq)
    for l in $locations ; do
        printf "location:    %s\n" $l
        cat $crash_types_file | grep $l | summarize_crashes
        printf "\n"
    done
}

process_crashes $crash_inventory
