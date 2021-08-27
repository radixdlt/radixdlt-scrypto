#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

# Set up environment
cargo run -- reset
cargo run -- new-account

# Test helloworld
package=`cargo run -- publish tests/helloworld.wasm | awk '/New package/ {print $NF}'`
component=`cargo run -- call-function $package Greeting new | awk '/New component:/ {print $NF}'`
cargo run -- call-method $component say_hello

# Test gumball machine
package=`cargo run -- publish tests/gumball-machine.wasm | awk '/New package/ {print $NF}'`
component=`cargo run -- call-function $package GumballMachine new | awk '/New gumball machine:/ {print $NF}'`
cargo run -- call-method $component get_gumball 1:01
