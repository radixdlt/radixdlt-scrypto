#!/bin/bash

set -x
set -e

REPORT_PATH=./target/coverage/html
BINARY_PATH=./target/release

if [ -d $REPORT_PATH ] ; then
    echo "Some coverage report already exists"
    exit
fi

# Collect code coverage data and generate report
grcov --source-dir .. --binary-path $BINARY_PATH --output-path $REPORT_PATH \
    --output-types html --branch --ignore-not-existing  \
    --excl-br-start "^declare_native_blueprint_state" --excl-br-stop "^}$" \
    --excl-start "^declare_native_blueprint_state" --excl-stop "^}$" \
    .

echo "Coverage report available at: $REPORT_PATH"
