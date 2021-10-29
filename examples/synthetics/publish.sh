#!/bin/bash

set -e
cd "$(dirname "$0")/../"

# Copies from reset_simulator.sh output

acc1_pub_key='04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9'
acc1_mint_auth='03d1f50010e4102d88aacc347711491f852c515134a9ecf67ba17c'


# SYNTHETICS

# mint SNX
snx=`rev2 new-resource-mutable $acc1_mint_auth --name "Synthetics Collateral Token" --symbol SNX --description "A token which is used in the synthetics component for collateral" | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
rev2 mint 117921786 $snx $acc1_mint_auth --signers $acc1_pub_key

# Publish synthetics blueprint
snythetics_blueprint=`rev2 publish ./synthetics | tee /dev/tty | awk '/Package:/ {print $NF}'`
snythetics_pool_creation_output=`rev2 call-function $snythetics_bp SyntheticPool new $po_cp $snx 4000000000 | tee /dev/tty | awk '/Component:|ResourceDef:/ {print $NF}'`
snythetics_pool_update_auth=`echo $snythetics_pool_creation_output | cut -d " " -f1`
snythetics_pool_component=`echo $snythetics_pool_creation_output | cut -d " " -f2`