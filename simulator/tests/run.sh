#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

rev2="cargo run --release --"

# Set up environment
$rev2 reset
$rev2 new-account

# Test helloworld
package=`$rev2 publish ../assets/helloworld.wasm | tee /dev/tty | awk '/New package/ {print $NF}'`
component=`$rev2 call-function $package Greeting new | tee /dev/tty | awk '/New component:/ {print $NF}'`
$rev2 call-method $component say_hello

# Test gumball machine
package=`$rev2 publish ../assets/gumball-machine.wasm | tee /dev/tty | awk '/New package/ {print $NF}'`
component=`$rev2 call-function $package GumballMachine new | tee /dev/tty | awk '/New gumball machine:/ {print $NF}'`
$rev2 call-method $component get_gumball 1:01
