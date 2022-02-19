#!/bin/bash

set -e
cd "$(dirname "$0")/../"
(../demo.sh)

#====================
# Set up environment
#====================

acc1_address='0293c502780e23621475989d707cd8128e4506362e5fed6ac0c00a'
acc1_pub_key='005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9'
acc1_mint_badge='031773788de8e4d2947d6592605302d4820ad060ceab06eb2d4711'
btc='03aedb7960d1f87dc25138f4cd101da6c98d57323478d53c5fb951'
usd='03de6e411593dcb3817187562c26c972cb024524f7b798f1c2980c'
snx='0334ce39f5112ef14ba6ed06c4085db6bd82feb98dcee9918c0359'
price_oracle_component='020bae515fc5ac81d75e5aba79f48ebe228e3d8411ee8a6d6bdea2'
price_oracle_update_auth='03f3855116a57c2a25c559d558da38b8500c3d96f0ee1277e3e41f'
synthetics_component='02c08be0470842c558144a6a91ddc2a947366db41c506f7057d74b'

#====================
# Test synthetics
#====================

# Create a Synthetics account
user1=`resim call-method $synthetics_component new_user | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`

# Stake 1000 SNX
vault_badge=`resim call-method $synthetics_component stake 1,$user1 1000,$snx | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim call-method $synthetics_component get_user_summary $user1

# Unstake 200 SNX
resim call-method $synthetics_component unstake 1,$user1 200
resim call-method $synthetics_component get_user_summary $user1

# Add sBTC synth
sbtc=`resim call-method $synthetics_component add_synthetic_token "BTC" $btc | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim call-method $synthetics_component get_user_summary $user1

# Mint 0.01 sBTC
resim call-method $synthetics_component mint 1,$user1 0.01 "BTC"
resim call-method $synthetics_component get_user_summary $user1

# Burn 0.005 sBTC
resim call-method $synthetics_component burn 1,$user1 0.005,$sbtc
resim call-method $synthetics_component get_user_summary $user1