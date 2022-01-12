# Day 10 - CoalYieldFarming
Received coal this Christmas and don't know what to do with it ? Stake it today and start earning more coal !

## How to test
1. Reset your environment: `resim reset`
1. Create the default account: `resim new-account`. Take note of the account's address.
1. Build and deploy the package on the ledger: `resim publish .`
1. Instantiate a component from the blueprint: `resim call-function [package_address] CoalYieldFarming new`. Store the component's address somewhere.
1. Request some Coal tokens from the faucet: `resim call-method [component_address] faucet`
1. Look at the resources in your account to know the Coal token's address: `resim show [account_address]`
1. Stake your coal tokens: `resim call-method [component_address] stake 1000,[coal_address]`. Take note of the returned ResourceDef, this is the badge allowing you to withdraw later.
1. Advance some epochs: `resim set-current-epoch 10`
1. Withdraw your staked tokens and reward: `resim call-method [component_address] withdraw 1,[staker_badge]`
1. Look at how many Coal tokens you have: `resim show [account_address]`.