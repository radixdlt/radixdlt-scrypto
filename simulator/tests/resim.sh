#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

cargo build --timings --bin resim
timings=$(basename $(ls -t target/cargo-timings/cargo-timing-* | head -n1))
mv target/cargo-timings/$timings target/cargo-timings/timings_resim_$timings

resim="cargo run --bin resim $@ --"

# Create test accounts and public keys
$resim reset
temp=`$resim new-account | awk '/Account component address:/ {print $NF}'`
account=`echo $temp | cut -d " " -f1`
account2=`$resim new-account | awk '/Account component address:/ {print $NF}'`

# Test - create fixed supply badge
minter_badge=`$resim new-badge-fixed 1 --name 'MinterBadge' | awk '/Resource:/ {print $NF}'`

# Test - create mutable supply token (requires a `ResourceAddress`)
token_address=`$resim new-token-mutable $minter_badge | awk '/Resource:/ {print $NF}'`

# Test - transfer non fungible
non_fungible_create_receipt=`$resim new-simple-badge --name 'TestNonFungible'`
non_fungible=`echo "$non_fungible_create_receipt" | awk '/NFAddress:/ {print $NF}'`
non_fungible_resource=`echo "$non_fungible_create_receipt" | awk '/Resource:/ {print $NF}'`
non_fungible_id=`echo "$non_fungible_create_receipt" | awk '/NFID:/ {print $NF}'`
# The below line looks like this: U32#1,resource_address
# You can put multiple ids into a bucket like so: String#Id1,String#num2,String#num3,resource_address
$resim call-method $account2 deposit "$non_fungible_id,$non_fungible_resource"

# Test - mint and transfer (Mintable that requires a `ResourceAddress`)
$resim mint 777 $token_address --proofs 1,$minter_badge
$resim transfer 111 $token_address $account2

# Test - publish, call-function and call-method and non-fungibles
owner_badge=`$resim new-simple-badge --name 'OwnerBadge' | awk '/NFAddress:/ {print $NF}'`
package=`$resim publish ../examples/hello-world --owner-badge $owner_badge | awk '/Package:/ {print $NF}'`
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
mkdir -p target
$resim new-badge-fixed 1 --name 'MintBadge' --manifest ./target/temp.rtm
cat ./target/temp.rtm
$resim publish ../examples/hello-world --owner-badge $owner_badge --manifest ./target/temp2.rtm
files=`ls target/*.blob`
blobs=`echo $files | sed 's/ / --blobs /g'`
$resim run ./target/temp2.rtm --blobs $blobs
$resim new-account --manifest ./target/temp3.rtm
$resim run ./target/temp3.rtm

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

$resim call-method $component organizational_authenticated_method --proofs 1,$supervisor_badge 1,$admin_badge 1,$superadmin_badge
$resim transfer 2 $token $account2 --proofs 1,$supervisor_badge 1,$admin_badge 1,$superadmin_badge
$resim mint 100000 $token --proofs 1,$supervisor_badge 1,$admin_badge 1,$superadmin_badge

# Test - publishing a large package
$resim publish ./tests/large_package.wasm --owner-badge $owner_badge

# Test - math types and numbers
$resim call-function $package "Numbers" test_input 1 2

# Test - set epoch
$resim set-current-epoch 100

# Test - create mutable supply token (requires a `NonFungibleAddress`)
non_fungible_create_receipt=`$resim new-simple-badge --name 'TestNonFungible'`
non_fungible=`echo "$non_fungible_create_receipt" | awk '/NFAddress:/ {print $NF}'`
non_fungible_resource=`echo "$non_fungible_create_receipt" | awk '/Resource:/ {print $NF}'`
non_fungible_id=`echo "$non_fungible_create_receipt" | awk '/NFID:/ {print $NF}'`

token_address=`$resim new-token-mutable $non_fungible | awk '/Resource:/ {print $NF}'`

# Test - mint and transfer (Mintable that requires a `NonFungibleAddress`)
$resim mint 777 $token_address --proofs "$non_fungible_id,$non_fungible_resource"
