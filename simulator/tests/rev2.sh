#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

rev2="cargo run --bin rev2 $@ --"

# Set up environment
$rev2 reset
account=`$rev2 new-account | tee /dev/tty | awk '/New account: / {print $NF}'`
$rev2 new-resource-fixed 333
resource=`$rev2 new-resource-mutable $account | tee /dev/tty | awk '/New resource: / {print $NF}'`
$rev2 mint-resource 666 $resource

# Test helloworld
package=`$rev2 publish ../assets/helloworld.wasm | tee /dev/tty | awk '/New package/ {print $NF}'`
component=`$rev2 call-function $package Greeting new | tee /dev/tty | awk '/Component:/ {print $NF}'`
$rev2 call-method $component say_hello

# Test gumball machine
package=`$rev2 publish ../assets/gumball-machine.wasm | tee /dev/tty | awk '/New package/ {print $NF}'`
component=`$rev2 call-function $package GumballMachine new | tee /dev/tty | awk '/Component:/ {print $NF}'`
$rev2 call-method $component get_gumball 1,01

# Export abi
$rev2 export-abi $package GumballMachine

# Show state
$rev2 show $package
$rev2 show $component
$rev2 show $account