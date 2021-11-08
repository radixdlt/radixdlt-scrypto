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
snx=`resim new-token-mutable $acc1_mint_auth --name "Synthetics Token" --symbol SNX --description "A token which is used in the synthetics component for collateral" | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim mint 114841533.01 $snx $acc1_mint_auth

# Publish synthetics blueprint
synthetics_blueprint=`resim publish ./synthetics | tee /dev/tty | awk '/Package:/ {print $NF}'`

# Publish SyntheticsPool with collat ratio of 4, using collateral of SNX and base price of USD
synthetics_component=`resim call-function $synthetics_blueprint SyntheticPool new $po_cp $snx $usd 4 | tee /dev/tty | awk '/Component:/ {print $NF}'`

# One SNX is $10.40
resim call-method $po_cp update_price $snx $usd 10.40  1,$po_update_auth

# One BTC is $66050.98
resim call-method $po_cp update_price $btc $usd 66050.98  1,$po_update_auth

## SYNTHETICS - TESTING

# Create a Synthetics account
user1=`resim call-method $synthetics_component new_user | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`

# Stake 1000 SNX
vault_badge=`resim call-method $synthetics_component stake 1,$user1 1000,$snx | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim call-method $synthetics_component get_user_summary $user1
#read -n 1 -p "Press any key to continue!"

# Unstake 200 SNX
resim call-method $synthetics_component unstake 1,$user1 200
resim call-method $synthetics_component get_user_summary $user1
#read -n 1 -p "Press any key to continue!"

# Add sBTC synth
sbtc=`resim call-method $synthetics_component add_synthetic_token "BTC" $btc | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim call-method $synthetics_component get_user_summary $user1
#read -n 1 -p "Press any key to continue!"

# Mint 0.01 sBTC
resim call-method $synthetics_component mint 1,$user1 0.01 "BTC"
resim call-method $synthetics_component get_user_summary $user1
#read -n 1 -p "Press any key to continue!"

# Burn 0.01 sBTC
resim call-method $synthetics_component burn 1,$user1 0.005,$sbtc
resim call-method $synthetics_component get_user_summary $user1
#read -n 1 -p "Press any key to continue!"

echo
echo "================================="
echo "SNX resource definition address: $snx"
echo "Synthetics blueprint address: $synthetics_blueprint, SyntheticPool"
echo "Synthetics component address: $synthetics_component"
echo "sBTC resource definition address: $sbtc"