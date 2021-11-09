#!/bin/bash

set -e
cd "$(dirname "$0")"

# reset database
resim reset

# new account
out=`resim new-account | tee /dev/tty | awk '/Component:|Public key:/ {print $NF}'`
acc1_address=`echo $out | cut -d " " -f1`
acc1_pub_key=`echo $out | cut -d " " -f2`
out=`resim new-account | tee /dev/tty | awk '/Component:|Public key:/ {print $NF}'`
acc2_address=`echo $out | cut -d " " -f1`
acc2_pub_key=`echo $out | cut -d " " -f2`
out=`resim new-account | tee /dev/tty | awk '/Component:|Public key:/ {print $NF}'`
acc3_address=`echo $out | cut -d " " -f1`
acc3_pub_key=`echo $out | cut -d " " -f2`
out=`resim new-account | tee /dev/tty | awk '/Component:|Public key:/ {print $NF}'`
acc4_address=`echo $out | cut -d " " -f1`
acc4_pub_key=`echo $out | cut -d " " -f2`

# generate acc1_minter_badge token
acc1_minter_badge=`resim new-badge-fixed --name MinterBadge 1 | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`

# mint btc
btc=`resim new-token-mutable $acc1_minter_badge --name Bitcoin --symbol BTC --description "Bitcoin is a decentralized digital currency, without a central bank or single administrator, that can be sent from user to user on the peer-to-peer bitcoin network without the need for intermediaries." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim mint 18843462 $btc $acc1_minter_badge

# mint ethereum
eth=`resim new-token-mutable $acc1_minter_badge --name Ethereum --symbol ETH --description "Ethereum is a decentralized, open-source blockchain with smart contract functionality." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim mint 117921786 $eth $acc1_minter_badge

# mint USD
usd=`resim new-token-mutable $acc1_minter_badge --name "US Dollar" --symbol USD --description "The United States dollar is the official currency of the United States and its territories." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim mint 19677000000000 $usd $acc1_minter_badge

# mint GBP
gbp=`resim new-token-mutable $acc1_minter_badge --name "Pound sterling" --symbol GBP --description "The pound sterling, known in some contexts simply as the pound or sterling, is the official currency of the United Kingdom, Jersey, Guernsey, the Isle of Man, Gibraltar, South Georgia and the South Sandwich Islands, the British Antarctic Territory, and Tristan da Cunha." | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resim mint 2896859000000 $gbp $acc1_minter_badge

# publish PriceOracle
price_oracle_package=`resim publish ./price-oracle | tee /dev/tty | awk '/Package:/ {print $NF}'`
out=`resim call-function $price_oracle_package PriceOracle new 18 1 | tee /dev/tty | awk '/Component:|ResourceDef:/ {print $NF}'`
price_oracle_update_auth=`echo $out | cut -d " " -f1`
price_oracle_component=`echo $out | cut -d " " -f2`

resim call-method $price_oracle_component update_price $btc $usd 57523 1,$price_oracle_update_auth
resim call-method $price_oracle_component update_price $eth $usd 3763 1,$price_oracle_update_auth
resim call-method $price_oracle_component update_price $btc $gbp 41950 1,$price_oracle_update_auth
resim call-method $price_oracle_component update_price $eth $gbp 2746 1,$price_oracle_update_auth
resim call-method $price_oracle_component update_price $btc $eth 15 1,$price_oracle_update_auth
resim call-method $price_oracle_component get_price $btc $eth
resim call-method $price_oracle_component get_price $eth $btc

# publish Radiswap
radiswap_package=`resim publish ./radiswap | tee /dev/tty | awk '/Package:/ {print $NF}'`

# publish AutoLend
auto_lend_package=`resim publish ./auto-lend | tee /dev/tty | awk '/Package:/ {print $NF}'`

# publish Synthetics
synthetics_package=`resim publish ./synthetics | tee /dev/tty | awk '/Package:/ {print $NF}'`

# Summary
echo "===================================================================================="
echo "Please assume a fixed number of decimal places for all resources: 18"
echo "Account 1 address: $acc1_address"
echo "Account 1 public key: $acc1_pub_key"
echo "Account 1 mint auth: $acc1_minter_badge"
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
echo "Price Oracle blueprint: $price_oracle_package PriceOracle"
echo "Price Oracle component: $price_oracle_component"
echo "Price Oracle update auth: $price_oracle_update_auth"
echo "Radixswap blueprint: $radiswap_package Radiswap"
echo "Synthetics blueprint: $auto_lend_package SyntheticPool"
echo "xPerpFutures blueprint: $synthetics_package ClearingHouse"
echo "===================================================================================="