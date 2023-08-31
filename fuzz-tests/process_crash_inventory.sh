#!/bin/bash

set -e
set -u
#set -x

crash_inventory=$1

#run_4895067287/afl/crash_summary.txt | awk -F'\\(| ' '($1 == "panic" && NF >= 16) { print $16}; ($1 == "panic" && NF < 16) { print "timeout"} ; $1 == "count" {print $5}' | paste - -  | awk '{a[$1]+=$2}END {for(x in a)print x"\t"a[x]}' | awk '{x += $2} END {print x}'
crash_types_file=$(dirname $crash_inventory)/crash_types.txt
function summarize_crashes {
    cat $crash_types_file | awk -F '\t' '
      {
          total+=$3
          printf "%-10s: %s\n%-10s: %s\n%-10s: %s\n\n", "panic", $1, "location", $2, "count", $3
      }
      END {
          printf "%-10s: %s\n", "Total", total
      }'
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
    #       panic   : thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: MismatchingMapKeyValueKind { key_value_kind: 2, actual_value_kind: 1 }', /home/radixdlt/_work/radixdlt-scrypto/radixdlt-scrypto/radix-engine-common/src/data/manifest/mod.rs:62:45
    #       count   : 2
    #       list    : 00b8778915d6dd5b50f2da62eb9d8c89054d5b0ad84f099f0a645ee85e7aa503.panic
    # - Timeout
    #       panic   :
    #       count   : 2
    #       list    : 01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b.panic

    # Generate intermediate file for easier further processing
    cat $crash_inventories | \
        awk '
            ($1 == "panic" && $0 ~ "panicked at") { for(i=3;i<NF;i++) printf $i" "; print "\n"$NF};
            ($1 == "panic" && !($0 ~ "panicked at")) { print "Timeout\nUnknown"};
            $1 == "count" {print $NF};
            $1 == "list" {print $NF}
            ' | \
        paste - - - - | sort -r -n -k2 > $crash_types_file

    # Summarize collected information
    cat $crash_types_file | summarize_crashes

}

process_crashes $crash_inventory
