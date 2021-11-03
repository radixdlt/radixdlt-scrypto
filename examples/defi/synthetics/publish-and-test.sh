#!/bin/bash

set -e

cd "$(dirname "$0")/../"
./demo.sh

# Copies from reset_simulator.sh output

acc1_address='02526629b90e1142492e934fbe807b446935407064db3ea2fcf856'
acc1_pub_key='04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9'
acc1_mint_auth='03d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c'
po_cp='0203672369abe1ac2f25e2a44ec60f8257172aac525030331cf2ea'
po_update_auth='03b6fe12281eb607ec48a4599f01a328db4836c1e3510b639d761f'
btc='03c29248a0d4c7d4da7b323adfeb4b4fbe811868eb637725ebb7c1'
usd='03806c33ab58c922240ce20a5b697546cc84aaecdf1b460a42c425'

resim set-default-account $acc1_address $acc1_pub_key

## SYNTHETICS - PREPARATION

# mint SNX
snx=`resim new-token-mutable $acc1_mint_auth --name "Synthetics Collateral Token" --symbol SNX --description "A token which is used in the synthetics component for collateral" | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim mint 117921786 $snx $acc1_mint_auth --signers $acc1_pub_key


# Publish synthetics blueprint
synthetics_blueprint=`resim publish ./synthetics | tee /dev/tty | awk '/Package:/ {print $NF}'`

# Publish SyntheticsPool with collat ratio of 4, using collateral of SNX and base price of USD
synthetics_pool_component=`resim call-function $synthetics_blueprint SyntheticPool new $po_cp $snx $usd 4 | tee /dev/tty | awk '/Component:/ {print $NF}'`

# One SNX is $42 (decimals in price oracle is 18)
resim call-method $po_cp update_price $snx $usd 42000000000000000000 1,$po_update_auth --signers $acc1_pub_key

# One TESLA is $1176 (decimals is 18)
resim call-method $synthetics_pool_component update_off_ledger_usd_price TESLA 1176000000000000000000

## SYNTHETICS - TESTING

# Stake some SNX tokens! (from the default account)
amount_to_stake=10
vault_badge=`resim call-method $synthetics_pool_component stake_to_new_vault "$amount_to_stake,$snx" --signers $acc1_pub_key | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`

# Top up our account
additional_amount_to_stake=21
resim call-method $synthetics_pool_component stake_to_existing_vault "1,$vault_badge" "$additional_amount_to_stake,$snx" --signers $acc1_pub_key

# Check our staked balance is 31
echo "There should be a line under results here saying Ok(Some(31)), in line with the CallMethod instruction"
resim call-method $synthetics_pool_component get_staked_balance "1,$vault_badge" --signers $acc1_pub_key

# Unstake 20 tokens
resim call-method $synthetics_pool_component unstake_from_vault "1,$vault_badge" 20 --signers $acc1_pub_key

# This should error because we have 11 tokens left
# resim call-method $synthetics_pool_component dispose_badge "1,$vault_badge" --signers $acc1_pub_key

resim call-method $synthetics_pool_component unstake_from_vault "1,$vault_badge" 11 --signers $acc1_pub_key

# Can now dispose badge as vault is empty
resim call-method $synthetics_pool_component dispose_badge "1,$vault_badge" --signers $acc1_pub_key

# Create new account (with a replacement badge!)
# Stake 1000 SNX = $42 000 of collateral (can mint $10 500 of synths)
amount_to_stake=1000
vault_badge=`resim call-method $synthetics_pool_component stake_to_new_vault "$amount_to_stake,$snx" --signers $acc1_pub_key | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`

# Mint 20 sTESLA which is 20 * $1176 = $23 520 - should fail
# resim call-method $synthetics_pool_component mint_synthetic "1,$vault_badge" TESLA 20 --signers $acc1_pub_key

# Mint 5 sTESLA which is 5 * $1176 and then 2 * $1176 for total of 7 * $1176 =  just within our collat ratio
resim call-method $synthetics_pool_component mint_synthetic "1,$vault_badge" TESLA 5 --signers $acc1_pub_key
resim call-method $synthetics_pool_component mint_synthetic "1,$vault_badge" TESLA 2 --signers $acc1_pub_key

# Check we have our 7 sTESLA
resim show $acc1_address


echo
echo "================================="
echo "SNX Resource Def Address: $snx"
echo "Synthetics Blueprint Address: $synthetics_blueprint"
echo "Synthetics Component Address: $synthetics_pool_component"
echo "Vault badge resource def: $vault_badge"