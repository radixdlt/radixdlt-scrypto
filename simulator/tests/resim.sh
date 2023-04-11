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

# Test - set epoch & time
$resim set-current-epoch 858585
$resim set-current-time 2023-01-27T13:01:16Z
$resim show-configs
ledger_state=`$resim show-ledger`
if [[ ${ledger_state} != *"858585"* ]];then
    echo "Epoch not set!"
    exit 1
fi
if [[ ${ledger_state} != *"2023-01-27T13:01:00Z"* ]];then
    echo "Time not set!"
    exit 1
fi

# Test - show account
# FIXME: renable after showing resource metadata in component dump
# account_dump=`$resim show $account`
# if [[ ${account_dump} != *"XRD"* ]];then
#     echo "XRD not present!"
#     exit 1
# fi
# if [[ ${account_dump} != *"Owner Badge"* ]];then
#     echo "Owner badge not present!"
#     exit 1
# fi

# Test - create fixed supply badge
minter_badge=`$resim new-badge-fixed 1 --name 'MinterBadge' | awk '/Resource:/ {print $NF}'`

# Test - create mutable supply token (requires a `ResourceAddress`)
token_address=`$resim new-token-mutable $minter_badge | awk '/Resource:/ {print $NF}'`

# Test - transfer non fungible
non_fungible_create_receipt=`$resim new-simple-badge --name 'TestNonFungible'`
non_fungible_global_id=`echo "$non_fungible_create_receipt" | awk '/NonFungibleGlobalId:/ {print $NF}'`
$resim call-method $account2 deposit "$non_fungible_global_id"

# Test - mint and transfer (Mintable that requires a `ResourceAddress`)
$resim mint 777 $token_address --proofs $minter_badge:1
$resim transfer 111 $token_address $account2

# Test - publish, call-function and call-method and non-fungibles
owner_badge=`$resim new-simple-badge --name 'OwnerBadge' | awk '/NonFungibleGlobalId:/ {print $NF}'`
package=`$resim publish ../examples/hello-world --owner-badge $owner_badge | awk '/Package:/ {print $NF}'`
component=`$resim call-function $package Hello instantiate_hello | awk '/Component:/ {print $NF}'`
$resim call-method $component free_token

# Test - export schema
$resim export-schema $package target/temp.schema

# Test - dump component state
$resim show $package
$resim show $component
$resim show $account
$resim show $account2
$resim show $token_address

# Test - output manifest
mkdir -p target
$resim new-badge-fixed 1 --name 'MintBadge' --manifest ./target/temp.rtm
cat ./target/temp.rtm
$resim publish ../examples/hello-world --owner-badge $owner_badge --manifest ./target/temp2.rtm
files=`ls target/*.blob`
blobs=`echo $files | sed 's/ / --blobs /g'`
$resim run ./target/temp2.rtm --blobs $blobs
$resim new-account --manifest ./target/temp3.rtm
# FIXME: temporarily commenting below call since it causes following panic.
#  thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', src/resim/mod.rs:395:26
#  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
echo "FIXME: reenable this call: " $resim run ./target/temp3.rtm

# Test - run manifest with a given set of signing keys
$resim generate-key-pair
$resim run ./target/temp2.rtm --blobs $blobs

# Test - nft
package=`$resim publish ./tests/blueprints --owner-badge $owner_badge | awk '/Package:/ {print $NF}'`
$resim call-function $package Foo nfts
$resim show $account

# Test - proofs
proofs_receipt=`$resim call-function $package Proofs new`

component=`echo "$proofs_receipt" | awk '/Component:/ {print $NF}'`

resources=`echo "$proofs_receipt" | awk '/Resource:/ {print $NF}'`
supervisor_badge=`echo $resources | cut -d " " -f1`
admin_badge=`echo $resources | cut -d " " -f2`
superadmin_badge=`echo $resources | cut -d " " -f3`
token=`echo $resources | cut -d " " -f4`

$resim call-method $component organizational_authenticated_method --proofs $supervisor_badge:1 $admin_badge:1 $superadmin_badge:1
$resim transfer 2 $token $account2 --proofs $supervisor_badge:1 $admin_badge:1 $superadmin_badge:1
$resim mint 100000 $token --proofs $supervisor_badge:1 $admin_badge:1 $superadmin_badge:1

# Test - math types and numbers
$resim call-function $package "Numbers" test_input 1 2

# Test - set epoch
$resim set-current-epoch 100

# Test - create mutable supply token (requires a `NonFungibleGlobalId`)
non_fungible_create_receipt=`$resim new-simple-badge --name 'TestNonFungible'`
non_fungible_global_id=`echo "$non_fungible_create_receipt" | awk '/NonFungibleGlobalId:/ {print $NF}'`

token_address=`$resim new-token-mutable $non_fungible_global_id | awk '/Resource:/ {print $NF}'`

# Test - mint and transfer (Mintable that requires a `NonFungibleGlobalId`)
$resim mint 777 $token_address --proofs "$non_fungible_global_id"
