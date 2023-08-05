#!/bin/bash

# Run this script from main radixdlt-scrypto repo folder

qemu_app=/home/ubuntu/qemu/qemu-x86_64
qemu_plugin=/home/ubuntu/qemu/libscrypto-qemu-plugin.so
qemu_cpu="Westmere-v1"

if [ -z "$*" ]; then echo "Provide path to folder with binaries to execute."; exit; fi

list=`find $1 -type f ! -name "*.*"`
idx=1

# count executable files
list_count=0
for i in $list; do [ -x $i ] && list_count=$((list_count+1)); done

for i in $list; do
    if [ -x $i ]; then
        echo "Running $idx/$list_count: $i"
        $qemu_app -cpu $qemu_cpu -plugin $qemu_plugin -d plugin -D log.txt $i --show-output --test --test-threads 1 > out.txt
        idx=$((idx+1))
    fi
done

# To generete results run command:
#  ./radix-engine-profiling/resources-tracker-macro/scripts/convert.py /tmp/scrypto-resources-usage
