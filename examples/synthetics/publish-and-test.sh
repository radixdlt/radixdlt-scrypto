#!/bin/bash

set -e

cd "$(dirname "$0")/../"
./reset_simulator.sh

# Copies from reset_simulator.sh output

acc1_address='02526629b90e1142492e934fbe807b446935407064db3ea2fcf856'
acc1_pub_key='04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9'
acc1_mint_auth='03d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c'
po_cp='0203672369abe1ac2f25e2a44ec60f8257172aac525030331cf2ea'
btc='03c29248a0d4c7d4da7b323adfeb4b4fbe811868eb637725ebb7c1'

rev2 set-default-account $acc1_address

# SYNTHETICS

# mint SNX
snx=`rev2 new-resource-mutable $acc1_mint_auth --name "Synthetics Collateral Token" --symbol SNX --description "A token which is used in the synthetics component for collateral" | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
rev2 mint 117921786 $snx $acc1_mint_auth --signers $acc1_pub_key


# Publish synthetics blueprint
synthetics_blueprint=`rev2 publish ./synthetics | tee /dev/tty | awk '/Package:/ {print $NF}'`
synthetics_pool_component=`rev2 call-function $synthetics_blueprint SyntheticPool new $po_cp $snx 4000000000 | tee /dev/tty | awk '/Component:|ResourceDef:/ {print $NF}'`

# Stake some SNX tokens! (from the default account)
amount_to_stake=10
vault_badge=`rev2 call-method $synthetics_pool_component stake_to_new_vault "$amount_to_stake,$snx" --signers $acc1_pub_key | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`

echo "Vault badge resource def: $vault_badge"

# Top up our account
additional_amount_to_stake=21
rev2 call-method $synthetics_pool_component stake_to_existing_vault "1,$vault_badge" "$additional_amount_to_stake,$snx" --signers $acc1_pub_key

# Check our staked balance is 31
echo "There should be a line under results here saying Ok(Some(31)), in line with the CallMethod instruction"
rev2 call-method $synthetics_pool_component get_staked_balance "1,$vault_badge" --signers $acc1_pub_key

# Unstake 20 tokens
rev2 call-method $synthetics_pool_component unstake_from_vault "1,$vault_badge" 20 --signers $acc1_pub_key

# This should error because we have 11 tokens left
# rev2 call-method $synthetics_pool_component dispose_badge "1,$vault_badge" --signers $acc1_pub_key

rev2 call-method $synthetics_pool_component unstake_from_vault "1,$vault_badge" 11 --signers $acc1_pub_key

# Can now dispose badge as vault is empty
rev2 call-method $synthetics_pool_component dispose_badge "1,$vault_badge" --signers $acc1_pub_key

echo
echo "================================="
echo "SNX Resource Def Address: $snx"
echo "Synthetics Blueprint Address: $synthetics_blueprint"
echo "Synthetics Component Address: $synthetics_pool_component"
echo "Vault badge resource def: $vault_badge"