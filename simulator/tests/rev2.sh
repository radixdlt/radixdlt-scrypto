#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

rev2="cargo run --bin rev2 $@ --"

# Set up environment
$rev2 reset
account=`$rev2 new-account | tee /dev/tty | awk '/Component:/ {print $NF}'`
account2=`$rev2 new-account | tee /dev/tty | awk '/Component:/ {print $NF}'`
$rev2 new-resource-fixed 333
resource_def=`$rev2 new-resource-mutable | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
$rev2 mint 777 $resource_def
$rev2 transfer 111 $resource_def $account2

# Test helloworld
package=`$rev2 publish ../examples/helloworld | tee /dev/tty | awk '/Package:/ {print $NF}'`
component=`$rev2 call-function $package Hello new | tee /dev/tty | awk '/Component:/ {print $NF}'`
$rev2 call-method $component free_token

# Test gumball machine
package=`$rev2 publish ../examples/gumball-machine | tee /dev/tty | awk '/Package:/ {print $NF}'`
component=`$rev2 call-function $package GumballMachine new | tee /dev/tty | awk '/Component:/ {print $NF}'`
$rev2 call-method $component get_gumball 1,030000000000000000000000000000000000000000000000000000

# Export abi
$rev2 export-abi $package GumballMachine

# Show state
$rev2 show $package
$rev2 show $component
$rev2 show $account
$rev2 show $account2