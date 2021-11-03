#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

# Set up environment
$resim reset
temp=`$resim new-account | tee /dev/tty | awk '/Component:|Public key:/ {print $NF}'`
account=`echo $temp | cut -d " " -f1`
account_key=`echo $temp | cut -d " " -f2`
account2=`$resim new-account | tee /dev/tty | awk '/Component:/ {print $NF}'`
minter_badge=`$resim new-badge-fixed 1 --name 'MintBadge' | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resource_def=`$resim new-token-mutable $minter_badge | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
$resim mint 777 $resource_def $minter_badge --signers $account_key
$resim transfer 111 $resource_def $account2 --signers $account_key

# Test helloworld
package=`$resim publish ../examples/helloworld | tee /dev/tty | awk '/Package:/ {print $NF}'`
component=`$resim call-function $package Hello new | tee /dev/tty | awk '/Component:/ {print $NF}'`
$resim call-method $component free_token

# Test gumball machine
package=`$resim publish ../examples/gumball-machine | tee /dev/tty | awk '/Package:/ {print $NF}'`
component=`$resim call-function $package GumballMachine new | tee /dev/tty | awk '/Component:/ {print $NF}'`
$resim call-method $component get_gumball 1,030000000000000000000000000000000000000000000000000004 --signers $account_key

# Test cross component call
$resim publish ../examples/gumball-machine --address 01a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876
package=`$resim publish ../examples/cross-component-call | tee /dev/tty | awk '/Package:/ {print $NF}'`
component=`$resim call-function $package Vendor new | tee /dev/tty | awk '/Component:/ {print $NF}' | tail -n1`
$resim call-method $component get_gumball 1,030000000000000000000000000000000000000000000000000004 --signers $account_key

# Export abi
$resim export-abi $package Vendor

# Show state
$resim show $package
$resim show $component
$resim show $account
$resim show $account2