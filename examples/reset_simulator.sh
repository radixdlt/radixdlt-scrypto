#!/bin/bash

set -e
cd "$(dirname "$0")"

# reset database
rev2 reset

# new account
out=`rev2 new-account | tee /dev/tty | awk '/Component:|Public key:/ {print $NF}'`
acc1_address=`echo $out | cut -d " " -f1`
acc1_pub_key=`echo $out | cut -d " " -f2`
out=`rev2 new-account | tee /dev/tty | awk '/Component:|Public key:/ {print $NF}'`
acc2_address=`echo $out | cut -d " " -f1`
acc2_pub_key=`echo $out | cut -d " " -f2`
out=`rev2 new-account | tee /dev/tty | awk '/Component:|Public key:/ {print $NF}'`
acc3_address=`echo $out | cut -d " " -f1`
acc3_pub_key=`echo $out | cut -d " " -f2`
out=`rev2 new-account | tee /dev/tty | awk '/Component:|Public key:/ {print $NF}'`
acc4_address=`echo $out | cut -d " " -f1`
acc4_pub_key=`echo $out | cut -d " " -f2`

# generate acc1_mint_auth token
acc1_mint_auth=`rev2 new-resource-fixed --name Acc1_mint_authToken 1 | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`

# mint btc
btc=`rev2 new-resource-mutable $acc1_mint_auth --name Bitcoin --symbol BTC --description "Bitcoin is a decentralized digital currency, without a central bank or single administrator, that can be sent from user to user on the peer-to-peer bitcoin network without the need for intermediaries." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
rev2 mint 18843462000000000000000000 $btc $acc1_mint_auth --signers $acc1_pub_key

# mint ethereum
eth=`rev2 new-resource-mutable $acc1_mint_auth --name Ethereum --symbol ETH --description "Ethereum is a decentralized, open-source blockchain with smart contract functionality." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
rev2 mint 117921786000000000000000000 $eth $acc1_mint_auth --signers $acc1_pub_key

# mint USD
usd=`rev2 new-resource-mutable $acc1_mint_auth --name "US Dollar" --symbol USD --description "The United States dollar is the official currency of the United States and its territories." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
rev2 mint 19677000000000000000000000000000 $usd $acc1_mint_auth --signers $acc1_pub_key

# mint GBP
gbp=`rev2 new-resource-mutable $acc1_mint_auth --name "Pound sterling" --symbol GBP --description "The pound sterling, known in some contexts simply as the pound or sterling, is the official currency of the United Kingdom, Jersey, Guernsey, the Isle of Man, Gibraltar, South Georgia and the South Sandwich Islands, the British Antarctic Territory, and Tristan da Cunha." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
rev2 mint 2896859000000000000000000000000 $gbp $acc1_mint_auth --signers $acc1_pub_key

# publish radiswap blueprint
rs_bp=`rev2 publish ./radiswap | tee /dev/tty | awk '/Package:/ {print $NF}'`

# publish price oracle blueprint
po_bp=`rev2 publish ./price-oracle | tee /dev/tty | awk '/Package:/ {print $NF}'`
out=`rev2 call-function $po_bp PriceOracle new 18 1 | tee /dev/tty | awk '/Component:|ResourceDef:/ {print $NF}'`
po_update_auth=`echo $out | cut -d " " -f1`
po_cp=`echo $out | cut -d " " -f2`

rev2 call-method $po_cp update_price $btc $usd 57523000000000000000000 1,$po_update_auth --signers $acc1_pub_key
rev2 call-method $po_cp update_price $eth $usd 3763000000000000000000 1,$po_update_auth --signers $acc1_pub_key
rev2 call-method $po_cp update_price $btc $gbp 41950000000000000000000 1,$po_update_auth --signers $acc1_pub_key
rev2 call-method $po_cp update_price $eth $gbp 2746000000000000000000 1,$po_update_auth --signers $acc1_pub_key
rev2 call-method $po_cp update_price $btc $eth 15000000000000000000 1,$po_update_auth --signers $acc1_pub_key
rev2 call-method $po_cp get_price $btc $eth
rev2 call-method $po_cp get_price $eth $btc

# Summary
echo "===================================================================================="
echo "Please assume a fixed number of decimal places for all resources: 18"
echo "Account 1 address: $acc1_address"
echo "Account 1 public key: $acc1_pub_key"
echo "Account 1 mint auth: $acc1_mint_auth"
echo "Account 2 address: $acc2_address"
echo "Account 2 public key: $acc2_pub_key"
echo "Account 3 address: $acc3_address"
echo "Account 3 public key: $acc3_pub_key"
echo "Account 4 address: $acc4_address"
echo "Account 4 public key: $acc4_pub_key"
echo "BTC: $btc"
echo "ETH: $eth"
echo "USD: $usd"
echo "GBP: $gbp"
echo "Radix swap blueprint: $rs_bp Radiswap"
echo "Price oracle blueprint: $po_bp PriceOracle"
echo "Price oracle component: $po_cp"
echo "===================================================================================="