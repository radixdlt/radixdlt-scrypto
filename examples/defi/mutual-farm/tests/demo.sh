#!/bin/bash

set -e
cd "$(dirname "$0")/../"
(../demo.sh)

#====================
# Set up environment
#====================

acc1_address='02526629b90e1142492e934fbe807b446935407064db3ea2fcf856'
acc1_pub_key='04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9'
acc1_minter_badge='03d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c'
btc='03c29248a0d4c7d4da7b323adfeb4b4fbe811868eb637725ebb7c1'
usd='03806c33ab58c922240ce20a5b697546cc84aaecdf1b460a42c425'
xrd='0373274042a2b57d3bcb44d907d9150d5e8f9e237bb58d5a4adbc0'
snx='03b6fe12281eb607ec48a4599f01a328db4836c1e3510b639d761f'
price_oracle_component='022cf5de8153aaf56ee81c032fb06c7fde0a1dc2389040d651dfc2'
price_oracle_update_auth='034ef4ca57d3a6846c2d757d475dbec8e3ae869b900dd8566073a4'
auto_lend_component='02517ccd96392dfbea25ef012ae5001f3d0994ad1a5d113157a02d'
synthetic_pool_component='0225267e74b1a067a09cdde372380c6e385d890c194359cb7c866d'

#====================
# Test mutual farm
#====================

# Create TESLA resource with no supply
tesla=`resim new-token-fixed --name "Tesla Token" --symbol "TESLA" 0 | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim call-method $price_oracle_component update_price $tesla $usd 1162.00  1,$price_oracle_update_auth
resim call-method $price_oracle_component update_price $xrd $snx 0.03901819  1,$price_oracle_update_auth

# Publish mutual farm package
mutual_farm_package=`resim publish . | tee /dev/tty | awk '/Package:/ {print $NF}'`

# Publish mutual_farm
resim call-function --trace $mutual_farm_package MutualFarm new $price_oracle_component $auto_lend_component $synthetic_pool_component "TESLA" $tesla 1000000,$xrd $snx 10
