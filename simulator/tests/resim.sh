#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

# Create test accounts and public keys
$resim reset
temp=`$resim new-account | awk '/Account component address:/ {print $NF}'`
account=`echo $temp | cut -d " " -f1`
account2=`$resim new-account | awk '/Account component address:/ {print $NF}'`

# Test - create fixed supply badge
minter_badge=`$resim new-badge-fixed 1 --name 'MintBadge' | awk '/Resource:/ {print $NF}'`

# Test - create mutable supply token
token_address=`$resim new-token-mutable $minter_badge | awk '/Resource:/ {print $NF}'`

# Test - mint and transfer
$resim mint 777 $token_address $minter_badge
$resim transfer 111 $token_address $account2

# Test - publish, call-funciton and call-method
package=`$resim publish ../examples/hello-world | awk '/Package:/ {print $NF}'`
component=`$resim call-function $package Hello instantiate_hello | awk '/Component:/ {print $NF}'`
$resim call-method $component free_token

# Test - export abi
$resim export-abi $package Hello

# Test - dump component state
$resim show $package
$resim show $component
$resim show $account
$resim show $account2
$resim show $token_address

# Test - output manifest
$resim new-badge-fixed 1 --name 'MintBadge' --manifest ./target/temp.rtm
cat ./target/temp.rtm
$resim publish ../examples/hello-world --manifest ./target/temp2.rtm
$resim run ./target/temp2.rtm
$resim new-account --manifest ./target/temp3.rtm
$resim run ./target/temp3.rtm

# Test - run manifest with a given set of signing keys
$resim generate-key-pair
$resim run ./target/temp2.rtm --signing-keys 4fc0db017bf9b80743b7151fee3f04bad817f2d8d7e34ae96c022fe7451b0ea3,329a27258d7e9496c42a110571e6ba0d47f2bda8bd610e9777d16853ab145b0c