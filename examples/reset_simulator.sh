#!/bin/bash

set -e
cd "$(dirname "$0")"

# reset database
rev2 reset

# new account
acc1=`rev2 new-account | tee /dev/tty | awk '/Component:/ {print $NF}'`
acc2=`rev2 new-account | tee /dev/tty | awk '/Component:/ {print $NF}'`
acc3=`rev2 new-account | tee /dev/tty | awk '/Component:/ {print $NF}'`
acc4=`rev2 new-account | tee /dev/tty | awk '/Component:/ {print $NF}'`

# mint btc
btc=`rev2 new-resource-mutable --name Bitcoin --symbol BTC --description "Bitcoin is a decentralized digital currency, without a central bank or single administrator, that can be sent from user to user on the peer-to-peer bitcoin network without the need for intermediaries." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
rev2 mint 18843462 $btc

# mint ethereum
eth=`rev2 new-resource-mutable --name Ethereum --symbol ETH --description "Ethereum is a decentralized, open-source blockchain with smart contract functionality." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
rev2 mint 117921786 $eth

# mint USD
usd=`rev2 new-resource-mutable --name "US Dollar" --symbol USD --description "The United States dollar is the official currency of the United States and its territories." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
rev2 mint 19677000000000 $usd

# mint GBP
gbp=`rev2 new-resource-mutable --name "Pound sterling" --symbol GBP --description "The pound sterling, known in some contexts simply as the pound or sterling, is the official currency of the United Kingdom, Jersey, Guernsey, the Isle of Man, Gibraltar, South Georgia and the South Sandwich Islands, the British Antarctic Territory, and Tristan da Cunha." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
rev2 mint 2896859000000 $gbp

# publish radiswap blueprint
rs_bp=`rev2 publish ./radiswap | tee /dev/tty | awk '/Package:/ {print $NF}'`

# publish price oracle blueprint
po_bp=`rev2 publish ./price-oracle | tee /dev/tty | awk '/Package:/ {print $NF}'`
po_cp=`rev2 call-function $po_bp PriceOracle new | tee /dev/tty | awk '/Component:/ {print $NF}'`
rev2 call-method $po_cp update_price $btc $usd 57523000000000
rev2 call-method $po_cp update_price $eth $usd 3763000000000
rev2 call-method $po_cp update_price $btc $gbp 41950000000000
rev2 call-method $po_cp update_price $eth $gbp 2746000000000
rev2 call-method $po_cp update_price $btc $eth 15000000000
rev2 call-method $po_cp get_price $btc $eth
rev2 call-method $po_cp get_price $eth $btc

# Summary
echo "===================================================================================="
echo "Account 1: $acc1"
echo "Account 2: $acc2"
echo "Account 3: $acc3"
echo "Account 4: $acc4"
echo "BTC: $btc"
echo "ETH: $eth"
echo "USD: $usd"
echo "GBP: $gbp"
echo "Radix swap blueprint: $rs_bp Radiswap"
echo "Price oracle blueprint: $po_bp PriceOracle"
echo "Price oracle component: $po_cp"
echo "===================================================================================="