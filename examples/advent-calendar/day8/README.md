# Day 8 - Multiple Components, One package
Today we will learn how to put multiple components inside a single Scrypto package. We will have two blueprint. The Santa component will call methods on the House component.

## How to test
1. Reset your environment: `resim reset`
1. Create the default account: `resim new-account`. Take note of the account's address.
1. Build and deploy the package to the ledger: `resim publish .`
1. Instantiate a new Santa component: `resim call-function [package_address] Santa new`. Take note of the last component's address.
1. Call the `go_into_house` method: `resim call-method [component_address] go_into_house 0`
1. Look at the balances of your account: `resim show [account_address]`. You should see that you now have cookies and milk.
1. Look at the balances of the Santa component: `resim show [component_address]`. You should see that the amount of gifts decreased.