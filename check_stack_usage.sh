#!/bin/bash

set -e

PACKAGE=radix-engine-tests
EXEC=arguments
TEST=vector_of_buckets_argument_should_succeed

# Return error if stack usage greater than
STACK_ERROR_VALUE=1572864
# Display warning if stack usage greater than
STACK_WARN_VALUE=1048576
stack=
output=$(mktemp)

# Running the test for debug variant only as it consumes stack way more greedy than release

function get_stack_usage() {
    echo Estimating stack usage...
    local low=10000
    local high=10000000
    while [ $low -lt $high ] ; do
        stack=$(( $low + ($high - $low) / 2))
        echo checking stack $stack

        if RUST_MIN_STACK=$stack cargo test -p $PACKAGE --test $EXEC -- $TEST >$output 2>&1 ; then
            if grep 'stack overflow' $output ; then
                cat $output
                echo "unexpected error occured"
                exit 1
            else
                high=$(( $stack - 1 ))
            fi
        else
            low=$(( $stack + 1 ))
        fi
    done
}

get_stack_usage
echo "Estimated debug stack usage $stack"
if [ $stack -ge $STACK_ERROR_VALUE ] ; then
    echo "ERROR - this is more than threshold $STACK_ERROR_VALUE! Please refer to stack_size.rs for more information, how to reduce stack usage"
    exit 1
elif [ $stack -ge $STACK_WARN_VALUE ] ; then
    echo "WARNING - this is more than threshold $STACK_WARN_VALUE. Please refer to stack_size.rs for more information, how to reduce stack usage"
fi

