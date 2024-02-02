#!/bin/bash

actual=$(du -sb $1 | awk '{ print $1 }')
limit=$(($2))

if [ $actual -gt $limit ]; then
    echo "File $1 is too large: actual = $actual, limit = $limit"
    exit 1
fi
