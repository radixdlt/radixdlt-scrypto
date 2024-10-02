#!/bin/bash

cd "$(dirname "$0")/../.."

qemu_app=/home/ubuntu/qemu/qemu-x86_64
qemu_plugin=/home/ubuntu/qemu/libscrypto-qemu-plugin.so
#qemu_cpu="Westmere-v1" # disabled due to some issue with missing CPU instructions in BLS implementation

if [ -z "$*" ]; then echo "Provide path to folder with binaries to execute."; exit; fi

list=`find "$1" -type f ! -name "*.*"`
idx=1

# count executable files
list_count=0
for i in $list; do [ -x $i ] && list_count=$((list_count+1)); done

for i in $list; do
    if [ -x $i ]; then
        echo "Running $idx/$list_count: $i"
        $qemu_app -plugin $qemu_plugin -d plugin -D log.txt $i --show-output --test --test-threads 1 > out.txt
        idx=$((idx+1))
    fi
done

# To generate results run command:
#  ./radix-engine-profiling-derive/scripts/convert.py /tmp/scrypto-resources-usage
