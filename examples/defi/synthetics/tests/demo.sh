#!/bin/bash

set -e
cd "$(dirname "$0")/../"
(../demo.sh)

#====================
# Set up environment
#====================

acc1_address='02526629b90e1142492e934fbe807b446935407064db3ea2fcf856'
acc1_pub_key='04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9'
acc1_mint_badge='03d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c'
btc='03c29248a0d4c7d4da7b323adfeb4b4fbe811868eb637725ebb7c1'
usd='03806c33ab58c922240ce20a5b697546cc84aaecdf1b460a42c425'
snx='03b6fe12281eb607ec48a4599f01a328db4836c1e3510b639d761f'
price_oracle_component='022cf5de8153aaf56ee81c032fb06c7fde0a1dc2389040d651dfc2'
price_oracle_update_auth='034ef4ca57d3a6846c2d757d475dbec8e3ae869b900dd8566073a4'
synthetics_component='0225267e74b1a067a09cdde372380c6e385d890c194359cb7c866d'

#====================
# Test synthetics
#====================

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

# Burn 0.005 sBTC
resim call-method $synthetics_component burn 1,$user1 0.005,$sbtc
resim call-method $synthetics_component get_user_summary $user1
#read -n 1 -p "Press any key to continue!"